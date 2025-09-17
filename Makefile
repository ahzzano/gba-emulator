counter fibonacci:
	arm-none-eabi-as -mcpu=arm7tdmi -EL -o test_files/$@.o test_files/$@.s
	arm-none-eabi-objcopy -O binary test_files/$@.o test_files/$@.gba
	mv test_files/$@.gba ./$@.gba

clean:
	rm test_files/*.o

