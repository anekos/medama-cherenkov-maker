/* Medama Cherenkov Maker
 * Copyright (C) 2019 anekos <anekos@snca.net>
 *
 * Supernova plug-in
 * Copyright (C) 1997 Eiichi Takamori <taka@ma1.seikyou.ne.jp>,
 *                    Spencer Kimball, Federico Mena Quintero
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program; if not, write to the Free Software
 * Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA 02111-1307, USA.
 */


use std::f64::consts::PI;


type SliceColor = [f64;3];
type TupleColor = (f64, f64, f64);

const FERROR: f64 = 0.000_001;


extern "C" {
    fn atan2(_: f64, _: f64) -> f64;
    fn rand() -> f64;
}


#[derive(Debug, Clone)]
pub struct Config {
    pub center: (f64, f64),
    pub color: TupleColor,
    pub n_spokes: usize,
    pub radius: f64,
    pub random_hue: f64,
}

#[cfg_attr(feature = "cargo-clippy", allow(many_single_char_names))]
pub fn nova(nv: &Config, pixels: &mut [u8], rowstride: i32, width: i32, height: i32, threads: usize) {
    let (cx, cy) = nv.center;
    let (cx, cy) = ((f64::from(width) * cx) as i32, (f64::from(height) * cy) as i32);
    let radius = clamp((f64::from(width * width + height * height)).sqrt() * nv.radius, 0.000_000_01, 100.0);

    let (spokes, spoke_colors) = {
        let mut spokes = vec![];
        let mut spoke_colors: Vec<SliceColor> = vec![];
        let (mut h, s, v) = rgb_to_hsv(nv.color);

        for _ in 0 .. nv.n_spokes {
            spokes.push(gauss());
            h += nv.random_hue / 360.0 * range_rand(-0.5, 0.5);

            if h < 0.0 {
                h += 1.0;
            } else if h >= 1.0 {
                h -= 1.0;
            }

            let rgb = hsv_to_rgb((h, s, v));
            spoke_colors.push(rgb);
        }

        (spokes, spoke_colors)
    };

    let mut lines: Vec<(usize, &mut [u8])> = pixels.chunks_mut(rowstride as usize).enumerate().collect();
    let chunks: Vec<&mut [(usize, &mut [u8])]> = lines.chunks_mut(height as usize / threads).collect();

    for chunk in chunks {
        for (y, line) in chunk {
            let y = *y as i32;
            for x in 0..width {
                let u = f64::from(x - cx) / radius;
                let v = f64::from(y - cy) / radius;
                let l = (u * u + v * v).sqrt();

                let t = (unsafe { atan2(u, v) } / (2.0 * PI) + 0.51) * nv.n_spokes as f64;
                let i = t.floor() as usize;
                let t = t - i as f64;
                let i = i % nv.n_spokes;

                let w1 = spokes[i] * (1.0 - t) + spokes[(i + 1) % nv.n_spokes] * t;
                let w1 = w1 * w1;

                let w = 1.0 / (l + 0.001) * 0.9;
                let nova_alpha = clamp(w, 0.0, 1.0);
                let compl_ratio = 1.0 - nova_alpha;
                let ptr = (x * 4 /* RGB+ALPHA */) as usize;

                for ci in 0..3 {
                    let in_color = f64::from(line[ptr + ci]) / 255.0;
                    let spoke_color = spoke_colors[i][ci] * (1.0 - t) + spoke_colors[(i + 1) % nv.n_spokes][ci] * t;

                    let mut out_color = if w > 1.0 {
                        clamp(spoke_color * w, 0.0, 1.0)
                    } else {
                        in_color * compl_ratio + spoke_color * nova_alpha
                    };

                    let c = clamp(w1 * w, 0.0, 1.0);
                    out_color += c;
                    out_color *= 255.0;
                    line[ptr + ci] = clamp(out_color, 0.0, 255.0) as u8;
                }
            }
        }
    }
}

fn gauss() -> f64 {
  let mut sum = 0.0;

  for _ in 0..6 {
      let r: f64 = unsafe { rand() };
      sum += r;
  }

  sum / 6.0
}

fn range_rand(from: f64, to: f64) -> f64 {
    let r = unsafe { rand() };
    r * (to - from) + from
}

#[cfg_attr(feature = "cargo-clippy", allow(many_single_char_names))]
fn rgb_to_hsv(rgb: TupleColor) -> TupleColor {
    let (r, g, b) = rgb;
    let max = max3(r, g, b);
    let min = min3(r, g, b);

    let mut h = max - min;

    if h > 0.0 {
        if feq(max, r, FERROR) {
            h = (g - b) / h;
            if h < 0.0 {
                h += 6.0
            }
        } else if feq(max, g, FERROR) {
            h = 2.0 + (b - r) / h
        } else {
            h = 4.0 + (r - g) / h
        }
    }
    h /= 6.0;

    let mut s = max - min;
    if max != 0.0 {
        s /= max;
    }

    (h, s, max)
}

static HRTBL: &'static [&'static [usize;3];6] = &[
    &[0, 3, 1],
    &[2, 0, 1],
    &[1, 0, 3],
    &[1, 2, 0],
    &[3, 1, 0],
    &[0, 1, 2]
];

#[cfg_attr(feature = "cargo-clippy", allow(many_single_char_names))]
fn hsv_to_rgb(hsv: TupleColor) -> SliceColor {
    let (h, s, v) = hsv;

    if s == 0.0 {
        return [v, v, v];
    }

    let h = h * 6.0;
    let i = h.floor();

    let f = h - i;
    let rs = &[v, v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f))];
    let idx = HRTBL[i as usize];

    [rs[idx[0]], rs[idx[1]], rs[idx[2]]]
}

fn clamp<T: PartialOrd>(v: T, from: T, to: T) -> T {
  if v < from {
      from
  } else if v > to {
      to
  } else {
      v
  }
}

#[inline]
pub fn feq(x: f64, y: f64, error: f64) -> bool {
    (x - y).abs() < error
}

#[inline]
pub fn max3(x: f64, y: f64, z: f64) -> f64 {
    if x < y {
        if y < z {
            z
        } else {
            y
        }
    } else {
        if x < z {
            z
        } else {
            x
        }
    }
}

#[inline]
pub fn min3(x: f64, y: f64, z: f64) -> f64 {
    if x < y {
        if x < z {
            x
        } else {
            z
        }
    } else {
        if y < z {
            y
        } else {
            z
        }
    }
}
