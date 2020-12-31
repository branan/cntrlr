MEMORY
{
        FLASH (rx) : ORIGIN = 0x00000000, LENGTH = 62K
        RAM  (rwx) : ORIGIN = 0x1FFFF800, LENGTH = 8K
}

EXTERN(__cntrlr_interrupts);
EXTERN(__cntrlr_exceptions);
EXTERN(__cntrlr_flash_configuration);

SECTIONS
{
        PROVIDE(__cntrlr_reset = __cntrlr_default_reset);

        .text :
        {
                . = 0;
                LONG(ORIGIN(RAM) + LENGTH(RAM))
                LONG(__cntrlr_reset)
                *(.__CNTRLR_EXCEPTIONS*)
                *(.__CNTRLR_INTERRUPTS*)
                . = 0x400;
                KEEP(*(.__CNTRLR_FLASH_CONFIG*))
                *(.text*)
        } > FLASH

        .rodata :
        {
                *(.rodata*)
                . = ALIGN(4);
        } > FLASH

        .data :
        {
                . = ALIGN(4);
                __cntrlr_data_start = .;
                *(.data*)
                . = ALIGN(4);
                __cntrlr_data_end = .;
        } > RAM AT>FLASH

        __cntrlr_data_flash_start = LOADADDR(.data);

        .bss :
        {
                . = ALIGN(4);
                __cntrlr_bss_start = .;
                *(.bss*)
                *(COMMON)
                . = ALIGN(4);
                __cntrlr_bss_end = .;
        } > RAM

        __cntrlr_heap_start = .;

        /DISCARD/ :
        {
                *(.ARM.exidx*);
                *(.ARM.extab*);
                *(.got*);
        }
}
