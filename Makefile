
trace=long-trace.dat

run:
	@ - rm mine.txt solution.txt mine-short.txt solution-short.txt
	@cargo build --release
	@echo "Running mine..."
	@./target/release/memory-hierarchy < $(trace) > mine.txt
	@echo "Running reference..."
	@ ./memhier_ref < $(trace) > solution.txt
	
	@echo `diff mine.txt solution.txt | grep "<" | wc -l` lines differ in outputs