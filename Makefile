trace=long-trace.dat

run:
	@ - rm test.txt reference.txt
	@cargo build --release
	@echo "Running mine..."
	@./target/release/memory-hierarchy < $(trace) > test.txt
	@echo "Running reference..."
	@ ./memhier_ref < $(trace) > reference.txt
	
	@echo `diff test.txt reference.txt | grep "<" | wc -l` lines differ in outputs