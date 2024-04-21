clean: cleanserver cleanapp
	echo "all clean."

cleanserver:
	rm -rf ./target/release/server*
	rm -rf ./target/debug/server*

cleanapp:
	rm -rf ./target/release/www/
	rm -rf ./target/debug/www/
	rm -rf ./app/dist/
	
build: buildserver buildapp
	echo "[RELEASE] all built."

builddebug: buildserverdebug buildappdebug
	echo "[DEBUG] all built."

buildapp:
	(cd ./app/ && trunk build --release && cd ..)
	mkdir ./target/release/www/
	(cp ./app/dist/* ./target/release/www/)

buildappdebug:
	(cd ./app/ && trunk build && cd ..)
	mkdir ./target/debug/www/
	(cp ./app/dist/* ./target/debug/www/)

buildserver:
	cargo build --bin server --release

buildserverdebug:
	cargo build --bin server
