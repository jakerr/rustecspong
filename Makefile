cgame: ./target/debug/librustecspong-28912547546b7996.a
	gcc -L/usr/local/homebrew/lib -l SDL2 -l freetype -l c -l m ./target/debug/librustecspong-28912547546b7996.a -o game hellors.c
