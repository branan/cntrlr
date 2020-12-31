MEMORY
{
        FLASH (rx) : ORIGIN = 0x20010000, LENGTH = 4032k
        RAM  (rwx) : ORIGIN = 0x80000000, LENGTH = 16k
}

EXTERN(__cntrlr_redv_reset)

SECTIONS
{
        PROVIDE(__cntrlr_reset = __cntrlr_default_reset);
        PROVIDE(__cntrlr_stack_top = ORIGIN(RAM) + LENGTH(RAM));

        .text :
        {
                . = ORIGIN(FLASH);
                *(.__CNTRLR_START*)
                *(.text*)
                . = ALIGN(64);
                *(.__CNTRLR_EARLY_TRAP*)
                . = ALIGN(64);
                *(.__CNTRLR_TRAP);
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
                *(.sbss*)
                *(COMMON)
                . = ALIGN(4);
                __cntrlr_bss_end = .;
        } > RAM

        __cntrlr_heap_start = .;

        /DISCARD/ :
        {
                *(.eh_frame*);
                *(.got*);
        }
}
