

diff:
	- rm mine.txt solution.txt mine-short.txt solution-short.txt
	cargo build --release
	# RUST_LOG=trace ./target/release/memory-hierarchy < write-trace.dat > mine.txt
	./target/release/memory-hierarchy < write-trace.dat > mine.txt
	./memhier_ref < write-trace.dat > solution.txt
	
	cat mine.txt | head -n10000 > mine-short.txt
	cat solution.txt | head -n10000 > solution-short.txt
	code --diff mine-short.txt solution-short.txt
	# code --diff mine.txt solution.txt
	# sleep 2
	# rm mine.txt solution.txt
