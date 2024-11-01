build-aarch64:
	cargo bundle --target aarch64-apple-darwin --release

build-app: create-icon build-aarch64

create-icon:
	mkdir icon.iconset
	sips -z 16 16     icon.png --out icon.iconset/icon_16x16.png
	sips -z 32 32     icon.png --out icon.iconset/icon_16x16@2x.png
	sips -z 32 32     icon.png --out icon.iconset/icon_32x32.png
	sips -z 64 64     icon.png --out icon.iconset/icon_32x32@2x.png
	sips -z 128 128   icon.png --out icon.iconset/icon_128x128.png
	sips -z 256 256   icon.png --out icon.iconset/icon_128x128@2x.png
	sips -z 256 256   icon.png --out icon.iconset/icon_256x256.png
	sips -z 512 512   icon.png --out icon.iconset/icon_256x256@2x.png
	sips -z 512 512   icon.png --out icon.iconset/icon_512x512.png
	sips -z 1024 1024 icon.png --out icon.iconset/icon_512x512@2x.png

	iconutil -c icns icon.iconset
	rm -R icon.iconset

create-dmg: build-app
	mkdir -p "tmp-dmg"
	cp -r "Frame Classifier.app" "tmp-dmg/"
	ln -s /Applications "tmp-dmg/Applications"
	hdiutil create -volname "Frame Classifier" -srcfolder "tmp-dmg" -ov -format UDZO "Frame Classifier.dmg"
	# Clean up
	rm -rf "tmp-dmg"
