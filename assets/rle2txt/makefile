CPPLANG =	-O2 -freg-struct-return
CPPWARN =	-Wall -Wextra -Wconversion -Wformat -Wmissing-noreturn \
		-Wno-long-long -Wold-style-cast -Wpadded -Wredundant-decls \
		-Wsign-promo -Wswitch-default -Wwrite-strings
CPPARCH =	-mconsole -m32 -march=i686 -mtune=i686 -mfpmath=387 -m128bit-long-double
CPPFLAGS =	$(CPPLANG) $(CPPWARN) $(CPPARCH)
LIBS =		-lsupc++ -lstdc++ -lm
7ZIP =		"c:\program files\7-Zip\7z.exe"

all:		rle2txt.exe rle2txtw.zip rle2txt.zip

rle2txt.exe:	rle2txt.o
		gcc -o $@ $+ $(LIBS)

rle2txt.o:	rle2txt.cpp
		gcc -c $(CPPFLAGS) -o $@ $<

rle2txt.zip:	makefile rle2txt.cpp rle2txt.txt
		$(7ZIP) u -uq0 -tzip -mx9 $@ makefile rle2txt.cpp rle2txt.txt

rle2txtw.zip:	makefile rle2txt.cpp rle2txt.exe rle2txt.txt
		$(7ZIP) u -uq0 -tzip -mx9 $@ makefile rle2txt.cpp rle2txt.exe rle2txt.txt

clean:
		rm rle2txt.o rle2txt.exe rle2txt.zip rle2txtw.zip

# End makefile
