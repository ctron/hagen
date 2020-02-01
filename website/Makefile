all: build

clean:
	rm -Rf output
	rm -Rf build
	rm -Rf node_modules

build/assets/jquery.min.js: node_modules/jquery/dist/jquery.min.js
	cp -p "$<" "$@"

build/assets/popper.min.js: node_modules/popper.js/dist/popper.min.js
	cp -p "$<" "$@"

build/assets/bootstrap/bootstrap.min.js: node_modules/bootstrap/dist/js/bootstrap.min.js
	cp -p "$<" "$@"

node_modules/bootstrap/scss/bootstrap.scss node_modules/popper.js/dist/popper.min.js:
	yarn install

build/assets/bootstrap/bootstrap.min.css: scss/custom.scss node_modules/bootstrap/scss/bootstrap.scss package.json postcss.config.js
	yarn css:compile
	yarn css:prefix
	yarn css:minify

assets: build/assets/bootstrap/bootstrap.min.css
assets: build/assets/bootstrap/bootstrap.min.js
assets: build/assets/jquery.min.js
assets: build/assets/popper.min.js

build: assets
	hagen

build-dev: assets
	hagen -b http://localhost:8080 -D

run: build-dev
	cd output && python3 -m http.server 8080

.PHONY: all build build-dev run assets clean
