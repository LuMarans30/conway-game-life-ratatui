// Convert Life pattern from .RLE format to text format
// (Runs on MinGW on Windows-32, but should easily run on other platforms)
// By Mark D. Niemiec 1997; updated and converted to c++ in 2013

#include <stdio.h>		// ferror fopen fprintf getc stderr putc ungetc



// .RLE files are ASCII text files with a one-line header,
// and followed by a variable-length run-length encoded image.
//
// The header may be preceded by any number of blank lines and/or
// comment lines beginning with a pound sign '#'.
//
// The header consists of several fields separated by commas,
// and ending in a newline.
// The first two fields are "x = nnn" and "y = nnn", (where nnn
// is an integer) respectively specifying the pattern width and height.
// These fields are required.
//
// An optional rule field, of the form "rule = Bnnn/Snnn" may follow,
// encoding totalistic rules.  By convention, this field is required
// a rule different from Life (B3/S23) is used.
//
// The data portion consists of one or more runs, terminated by
// an exclamation mark '!'.
// The runs are defined by a single character, preceded by an optional
// integer repeat factor (which defaults to 1 if omitted).
// The character may be one of the following:
//	$	Newline
//	b	Dead cell
//	o	Living cell, or first living state in multi-state Life
// Some programs use other letters to indicate different states.
// For example, Golly also uses . for dead cells, and different letters
// for different states in multi-state rules.
// Embedded white space is always ignored.
//
// The RLE standard adds the following additional stipulations:
// Lines should not exceed 70 characters to avoid confusing brain-damaged
// e-mail programs, so newlines should be added as required to ensure this.
// Newlines and other white-space may not be inserted in the middle of a
// repeat factor, nor between a character and its repeat factor.
// This program can, however, read files which violate these stipulations.
//
// This program can also read .RLE files without headers,
// since it does not use the header information for anything.



// Read and convert a RLE-format file

void ReadRle (FILE *srcFile, FILE *dstFile)
{
	int		c;		// character read from file

	for (;;) {			// strip leading comment lines:
		switch ((c = getc (srcFile))) {
		case EOF:			// end of file: no comment
			return;
		case '\n':			// blank line: ignored
			continue;
		case '#':			// #: comment line
			while ((c = getc (srcFile)) != '\n') {
				if (c == EOF) {	// EOF in comment!?
					return;
				}
			}
			continue;
		default:			// other: legitimate text
			ungetc (c, srcFile);
			break;
		}
		break;
	}

	while ((c = getc(srcFile)) == ' ' || c == '\t' || c == '\n') {
	}

	if (c == 'x') {			// header line is supplied (and ignored)
		while ((c = getc(srcFile)) != EOF && c != '\n') {
		}
	} else {			// no header: translator doesn't care
		ungetc (c, srcFile);
	}

	int		lastc = '\n';	// last character written
	int		n;		// repeat count

	for (n = 0; ; ) {
		while ((c = getc (srcFile)) == ' ' || c == '\t' || c == '\n') {
		}
		if (c >= '0' && c <= '9') {
			n = 10*n + (c-'0');
			continue;
		} else if (c == '!' || c == EOF) {	// End: flush last line
			if (lastc != '\n') {
				putc ('\n', dstFile);
			}
			return;
		}
		if (n == 0) {				// default = repeat once
			n = 1;
		}
		if (c == '$') {				// $ = newline
			c = '\n';
		} else if (c == 'b') {			// b = space;
			c = '.';			// show other states as-is
		}
		for (lastc = c; n > 0; --n) {		// output the run
			putc (c, dstFile);
		}
	}
}



// Command-line interface

int main (int argc, char **argv)
{
	if (argc != 3) {
		fprintf (stderr,
		  "Usage:  rle2txt infile.rle outfile.txt\n");
		return 1;
	}

	FILE		*srcFile = fopen (argv[1], "r");
	if (!srcFile) {
		fprintf (stderr, "Cannot open %s\n", argv[1]);
		return 1;
	}

	FILE		*dstFile = fopen (argv[2], "w");
	if (!dstFile) {
		fprintf (stderr, "Cannot create %s\n", argv[2]);
		return 1;
	}

	ReadRle (srcFile, dstFile);

	if (ferror (dstFile)) {
		fprintf (stderr, "Error writing to %s\n", argv[2]);
		return 1;
	}

	return 0;
}

// End rle2txt.cpp
