cgame: ./target/debug/librustecspong.a
	gcc -L/usr/local/homebrew/lib -L./target/debug/ -l SDL2 -l freetype -l c -l rustecspong -lm -o game hellors.c -v
