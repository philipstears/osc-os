ENTRY(kernel_main)

SECTIONS
{

  /*
   * x64, higher half start + 1MiB
   */
	. = 0xFFFF800000100000;

	_kernel_start = .;

	.text ALIGN (4K) :
	{
		*(.text)
	}

	.rodata ALIGN (4K) :
	{
		*(.rodata)
	}

	.data ALIGN (4K) :
	{
		*(.data)
	}

	.bss ALIGN (4K) :
	{
		*(.bss)
	}

  /DISCARD/ :
  {
    *(.note.gnu.*)
    *(.comment)
  }

	_kernel_end = .;
}
