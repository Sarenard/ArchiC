.PHONY: clean build

clean:
	rm -f IO/*.bin IO/*.asm

build:
	cargo run -- $(filter-out $@,$(MAKECMDGOALS))

%:
	@: