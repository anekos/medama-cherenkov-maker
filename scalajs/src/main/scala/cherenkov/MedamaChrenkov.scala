package cherenkov

import util.Random
import collection.mutable

import scala.scalajs.js
import js.{Array => JSArray, JSApp, Date}

import org.scalajs.dom
import dom.{Event, DragEvent, FileReader}
import dom.document
import dom.raw.{HTMLCanvasElement, HTMLImageElement, HTMLInputElement, CanvasRenderingContext2D, ImageData}



object MedamaCherenkov extends JSApp {
  val rand = new Random

  case class RGB(r: Double, g: Double, b: Double, alpha: Double = 0.0d) {
    val pixel: Array[Double] =
      Array(r, g, b)
  }

  case class HSV(h: Double, s: Double, v: Double, alpha: Double = 0.0d)

  object RGB {
    def fromArray(ary: Array[Double]): RGB =
      RGB(ary(0), ary(1), ary(2))
  }

  def gauss(): Double = {
    var sum = 0.0
    for (i <- 0 until 6)
      sum += rand.nextDouble
    sum / 6.0
  }

  def rangeRand(from: Double, to: Double): Double = {
    var d = to - from;
    return rand.nextDouble * d + from;
  }

  def rgb2hsv(rgb: RGB): HSV = {
    val v = Seq(rgb.r, rgb.g, rgb.b).max
    val min = Seq(rgb.r, rgb.g, rgb.b).min

    if (v == 0)
      return HSV(0, 0, 0)

    val d = v - min
    val s = d / v
    val rr = (v - rgb.r) / d
    val gg = (v - rgb.g) / d
    val bb = (v - rgb.b) / d

    val h = if (v == rgb.r) { bb - gg }
       else if (v == rgb.g) { 2 + rr - bb }
       else                 { 4 + gg - rr }

    HSV(
      h = (if (h < 0) h + 1 else h) / 6,
      s = s,
      v = v)
  }

  def hsv2rgb(hsv: HSV): RGB = {
    val hrTable = Array(
      Array(0, 3, 1),
      Array(2, 0, 1),
      Array(1, 0, 3),
      Array(1, 2, 0),
      Array(3, 1, 0),
      Array(0, 1, 2))

    if (hsv.s == 0)
      RGB(hsv.v, hsv.v, hsv.v)

    val hh: Double = hsv.h * 6
    val i: Int = hh.floor.toInt
    val f: Double = hh - i
    val rs = Array(
      hsv.v,
      hsv.v * (1 - hsv.s),
      hsv.v * (1 - hsv.s * f),
      hsv.v * (1 - hsv.s * (1 - f)))
    val idx = hrTable(i)

    RGB.fromArray(idx.map(rs))
  }

  def clamp(v: Double, from: Double, to: Double): Double =
    return if (v < from) from
      else if (v > to)   to
      else               v

  def time[T](name: String)(f: => T): T = {
    val atStart = new Date
    val result = f
    val t = (new Date().getTime - atStart.getTime).toDouble / 1000
    println(s"$name: $t msec")
    result
  }

  def nova(context: CanvasRenderingContext2D, xc: Int, yc: Int, width: Int, height: Int): Unit = {
    val nSpokes: Int = 100
    val randomHue: Int = 0
    val radius: Double = 20

    val imageData = context.getImageData(0, 0, width, height)
    val pixels: JSArray[Int]  = imageData.data

    val hsv = rgb2hsv(RGB(0, 0, 1.0, 1.0))

    val spoke: Array[Double] = (1 to nSpokes) map { _ => gauss() } toArray

    val spokeColor: Array[RGB] = {
      var h = hsv.h
      (1 to nSpokes) map { _ =>
        h += randomHue / 360.0 * rangeRand(-0.5, 0.5)

        if (h < 0)
          h += 1.0
        else if (h >= 1.0)
          h -= 1.0

        hsv2rgb(hsv.copy(h = h))
      }
    } .toArray

    for {
      y <- 0 until height
      x <- 0 until width
    } {
      val u: Double = (x - xc) / radius
      val v: Double = (y - yc) / radius
      val l: Double = math.sqrt(u * u + v * v)

      var t: Double = (math.atan2(u, v) / (2 * math.Pi) + 0.51) * nSpokes
      var i: Int = t.floor.toInt
      t -= i
      i %= nSpokes

      var w1: Double = spoke(i) * (1 - t) + spoke((i + 1) % nSpokes) * t
      w1 = w1 * w1

      var w: Double = 1 / (l + 0.001) * 0.9
      val ratio: Double = clamp(w, 0.0, 1.0)
      var compRatio: Double = 1.0 - ratio
      var ptr: Int = (y * width + x) * 4

      {
        spokeColor(i).pixel zip spokeColor((i + 1) % nSpokes).pixel
      } .zipWithIndex foreach { case ((col1, col2), ci) =>
        val srcCol: Double = pixels(ptr + ci).toDouble / 255
        val spokeCol: Double = col1 * (1.0 - t) + col2 * t;
        val outCol: Double = {
          {
            if (w > 1.0) clamp(spokeCol * w, 0.0, 1.0)
            else srcCol * compRatio + spokeCol * ratio
          } + clamp(w1 * w, 0.0, 1.0)
        } * 255

        pixels(ptr + ci) = clamp(outCol, 0, 255).toInt
      }

    }

    context.putImageData(imageData, 0, 0)
    println("OK")
  }

  def cancel(e: Event): Unit = {
    e.stopPropagation()
    e.preventDefault()
  }

  def main(): Unit = {
    val canvas = document.body.querySelector("#cherenkov").asInstanceOf[HTMLCanvasElement]
    val context = canvas.getContext("2d").asInstanceOf[CanvasRenderingContext2D]
    val image = document.getElementById("image").asInstanceOf[HTMLImageElement];
    val imageData = document.getElementById("image-data").asInstanceOf[HTMLInputElement]

    image.addEventListener("load", { (e: Event) =>
      canvas.width = image.width
      canvas.height = image.height

      context.fillStyle = "rgb(255, 255, 255)"
      context.fillRect(0, 0, 150, 150)
      context.drawImage(image, 0, 0)
    }, false)

    canvas.addEventListener("click", { (e: DragEvent) =>
      val target = e.target.asInstanceOf[HTMLCanvasElement]
      time("nova") {
        nova(
          context = context,
          xc = e.pageX.toInt - target.offsetLeft.toInt - 2,
          yc = e.pageY.toInt - target.offsetTop.toInt - 2,
          image.width,
          image.height)
      }
    }, false)

    canvas.addEventListener("dragenter", cancel _, false)
    canvas.addEventListener("dragover", cancel _, false)

    canvas.addEventListener("drop", { (e: DragEvent) =>
      cancel(e)

      val file = e.dataTransfer.files(0)

      {
        val reader = new FileReader

        reader.addEventListener("loadend", { (_: Event) =>
          image.src = reader.result.toString
        }, false)
        reader.readAsDataURL(file)
      }

      {
        val reader = new FileReader
        reader.addEventListener("loadend", { (_: Event) =>
          imageData.value = reader.result.toString.replace("^data:[^;]+;base64,", "")
        }, false)

        reader.readAsDataURL(file)
      }


    }, false)
  }
}
