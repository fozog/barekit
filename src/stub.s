/* 
    SPDX-License-Identifier: Mozilla Public License 2.0
    Copyrigth (c) 2022-2023 François-Frédéric Ozog
    
*/

.text

/* UEFI file format comes from Windows which has a MSDOS header
   before a PE header.
   The MSDOS headers starts wuth signature "MZ" and then has a number of fields.
   This header is mostly ignored except for the MZ and the PE header position.
   U-Boot may use some reserved words to identify an aarch64 program.
   As this MSDOS header is largely ignored, there was an idea to use this even
   in contexts that considers the .EXE file just binary and jumps to the first
   instruction. The trick is MZ is the coding of a set of add instructions
   that can be considered begnin if they dont touch initial registers.
   MZ\x00\x091 corresponds to an ARM instruction add x13, x18, #0x16.
   The next instruction, replacing a pair of shorts in the MSDOS header is a
   jump to actual entry point.

   The goal would have been to leverage the /STUB: link option to add this file as a prefix
   But it does not seem to work. 
   Prepending a larger that standard DOS header does not work as it would require
   changing some COFF values to reflect their new position in the file
  */


. = 0;
ImageBase:

//+0 DOS Header
	//WORD   e_magic;               // Magic number
	//WORD   e_cblp;                // Bytes on last page of file
	.ascii	"MZ"
	// the combination with previous MZ is equivalent to instruction add x13, x18, #0x#6
	.ascii "\x00\x091"

//+4 DOS Header
	//WORD   e_cp;                  // Pages in file
	//WORD   e_crlc;                // Relocations

// do not use a label to jump to the begining of the stub code
// strangely, the compilation result depends on the compiler
// and sometimes the displacement is not pre-calculated
// and results to a branch to self !
// the followinf form comoiles correctly on several combinations of
// compiler/os
	b	1f
	
//+8 DOS Header
	//WORD   e_cparhdr;             // Size of header in paragraphs
	//WORD   e_minalloc;            // Minimum extra paragraphs needed
	//WORD   e_maxalloc;            // Maximum extra paragraphs needed
	//WORD   e_ss;                  // Initial (relative) SS value
	.quad 	0						// for Linux Image this is text_offset (see https://elixir.bootlin.com/linux/latest/source/Documentation/arm64/booting.rst#L79)
//+16
	//WORD   e_sp;                  // Initial SP value
	//WORD   e_csum;                // Checksum
	//WORD   e_ip;                  // Initial IP value
	//WORD   e_cs;                  // Initial (relative) CS value
	.quad 	0						// for Linux Image this is image_size (see https://elixir.bootlin.com/linux/latest/source/Documentation/arm64/booting.rst#L80)
									// should be updated by replace_stub script at runtime.
									// if left to zero, u-boot assumes 16MB and text_offset = 
									// https://elixir.bootlin.com/u-boot/latest/source/arch/arm/lib/image.c#L53
//+24
	//WORD   e_lfarlc;               // File address of relocation table
	//WORD   e_ovno;                 // Overlay number
	//WORD   e_res[4];               // Reserved words
	.quad	0						// For Linux Image this is flags https://elixir.bootlin.com/linux/latest/source/Documentation/arm64/booting.rst#L106
									// . 0 -> Little Endian, unspeficied paging, 2MB aligned base should be as close as possible to the base of DRAM
//+32
	.short 0
//+34
	.short 0
//+36
	//WORD   e_oemid;                // OEM identifier (for e_oeminfo)
	.short 0
//+38
	//WORD   e_oeminfo;              // OEM information; e_oemid specific
	.short 0
//+40
	//WORD   e_res2[10];             // Reserved words
	.short 0
//+42
	.short 0
//+44
	.short 0
//+46
	.short 0
//+48
	.short 0
//+50
	.short 0
//+52
	.short 0
//+54
	.short 0
//+56
	.ascii "ARM\x64"		// https://elixir.bootlin.com/u-boot/latest/source/arch/arm/lib/image.c#L43
							// Trick to get  U-boot to recognize this as an Arm Image when booting with booti
//+60
	//LONG   e_lfanew;      // File address of new exe header
	.long	pe_header - ImageBase				// Offset to the PE header.


//+64;
1:
	// when entering "raw" (i.e. baremetal, BL32 or BL33 TFA payload)
	// the execution will start at offset 0 and continue here.
	// x0 is set by the loading entity to FDT pointer except for BL32 where it is set to 0
	// let's make sure that
	// - x1 is 0
	// - x2 points to load address
	// - x3 to end of image
	
		mov		x1,xzr
        adrp    x2, 0
		
        ldr     w10, [x2, #0x3C]
        add     x10, x10, x2
    // x10 now holds absolute address of PE header
        ldr     w11, [x10, #40]         //AddressOfEntryPoint
        add     x11, x11, x2

	// place the stack at the end of image+SizeOfStackCommit
	// first: get the end of image in x4
    	ldr     w3, [x10, #80]         // SizeOfImage
        add     x3, x3, x2
	// then add the SizefOfStackReserve
        ldr     x12, [x10, #96]        // get SizefOfStackReserve
        add     sp, x3, x12
		
	// continue execution to the known entry point
	// x0 = FDT, x1=0, x2= Load address, x3 = end of image
        br      x11

	// loaded by an EFI entity, execution will start directly at the registered AddressOfEntryPoint
	// with x0 set to a UEFI handle and x1 to SystemTableAddress. It will not execute the above code.

pe_header:
