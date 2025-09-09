all: counter

counter:
	arm-none-eabi-as -mcpu=arm7tdmi -EL -o test_files/counter.o test_files/counter.s
	arm-none-eabi-objcopy -O binary test_files/counter.o test_files/counter.gba
	cp test_files/counter.gba ./counter.gba


