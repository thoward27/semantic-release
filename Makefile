
x86_64-unknown-linux-gnu:
	docker build -t semantic/x86_64-unknown-linux-gnu -f Dockerfiles/x86_64-unknown-linux-gnu Dockerfiles/
	cross build --release --target=x86_64-unknown-linux-gnu
	cd ./target/x86_64-unknown-linux-gnu && tar -czvf x86_64-unknown-linux-gnu.tar.gz release
