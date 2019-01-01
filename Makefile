

defualt:
	$(eval tempdir := $(shell mktemp -d))
	git submodule update --init --recursive
	git checkout-index -a --prefix $(tempdir)/
	git submodule foreach --recursive 'git checkout-index -a --prefix $(tempdir)/lib/wiv/'
	aws --profile cherenkov-updater s3 sync $(tempdir) s3://cherenkov.snca.net/
	tree $(tempdir)
