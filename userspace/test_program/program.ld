

PHDRS
{
  headers PT_PHDR PHDRS;
  text PT_LOAD FILEHDR PHDRS;
  data PT_LOAD;
  bss PT_LOAD;
}



SECTIONS
{
  . = SIZEOF_HEADERS;
  /* All text sections will go here */
  .text : {
      BYTE(0x33)
      *(.text.init)
      *(.text*)
  } : text
  .rodata : {
      *(.rodata)
      *(.rodata.*)
  } :data
  .data :
  {
      *(.sdata.*)
      *(.sdata)
      *(.data.*)
      *(.data)
      *(.pagetable)
  }:data

  .bss ALIGN (4K) : AT (ADDR (.bss))
  {
      *(COMMON)
      *(.sbss.*)
      *(.sbss)
      *(.bss.*)
      *(.bss)
      *(.bootstrap_stack)
  }
  .eh_frame_hdr ALIGN (4K) : AT (ADDR (.eh_frame_hdr))
  {
      *(.eh_frame_hdr)
  }
  .eh_frame ALIGN (4K) : AT (ADDR (.eh_frame))
  {
      *(.eh_frame)
  }
  . = ALIGN(4096);
  start = 0x2390;
  PROVIDE(_stack_end = .);
}
