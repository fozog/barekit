# Introduction

The produced file should be bootable as an EFI executable, as a Linux Image, 
or as a plain binary. 

As an EFI executable, the file need to abid to PE/COFF format, with sections,
startup address and relocations. The entity loading it will verify the overall
format, place sections in memory as per the requested layout, apply relocations,
 and jump to the startup location.

As a Linux Image, the file need to abid to some limited format. The entiry
loading it, will just read the file as is in memory and jump to offset 0.

As a plain binary, the entity loading the file makes no asumptions about the
format and just jumps to offset 0.

The good news is that Linux proved that PE/COFF and Linux Image formats can be
"melted".
For the PE/COFF: the first (most of the time 0x78) bytes are entirely ignored
by EFI loaders as it corresponds to a "DOS Stub": for historic reasons, PE/COFF
was used by first version of Windows and Microsoft wanted those executables not
to cause trouble on DOS machines. The stub contains an MSDOS executable that
was just displaying a message "can't run on DOS" !. to be precise, the first 
0x40 bytes are the DOS Hheader and bytes 0x40-0x77 are the executable.
EFI loaders care only about the first two bytes ("MZ") and a 32 bits offset at file
position 0x3C to discover where is the begining of the PE header.
For the Linux Image, the only check is the byte sequence "ARM\x64" at offset 56!
Ignoring such a large quantity of bytes of the PE/COFF and the Linux Image
allow the "melting".
So a "melted" file contains "MZ" at offset 0, "ARM\x64" at offset 56, and
PE Offset (usually 0x78) at offset 60. the rest of the bytes up to 0x78 can be changed.
In addition to format compliance, startup need to be handled. The startup location
of a PE/COFF is located in a PE/COFF field while it is assumed to be 0 for a
Linux Image. Hapilly, the PE/COFF format that starts with "MZ" 16 bits quantity
which can be part of an "add" 32 bits instruction that is benign. So a program
can be placed from bytes 0x04 to 0x37 and 0x40 to 0x77. What Linux does is
place a "branch" instruction at offset 0x4 that jumps to 0x40 and place a stub
at offset 0x40-0x77.

# barekit stub

In a perfect world, the linker option /STUB:<path_to_stub.exe> should work
and thus it should be fairly simple to replace the DOS stub by a custom one.
But that would be too simple...

So it is needed to create the stub and replace the default one.
I wanted to code to run on both Linux and MacOS, so producing that stub became
a little bit more complex because objcopy is not available on MacOS.
The src/stub.s is compiled to a ".o" file and the binary is extracted with 
extract_text.sh. This causes pratical impossiblity to refer to extern symbols.
So the stub parses the PE/COFF header to find the right location to start.
Replacement of the DOS Stub by the barekit stub is done by replace_stub.sh
