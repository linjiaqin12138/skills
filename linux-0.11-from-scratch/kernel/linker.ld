/* Limine 会把内核映射到虚拟地址 0xFFFFFFFF80000000 起的高半区，
 * 链接地址必须与之一致（见 PROTOCOL.md "Caching / Machine State at Entry"）。
 */
OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start)

SECTIONS
{
    . = 0xFFFFFFFF80000000;

    .text : {
        *(.text .text.*)
    }

    /* 段间按页对齐，让 lld 为不同权限的段生成独立的 PT_LOAD */
    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .rodata : {
        *(.rodata .rodata.*)
    }

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    /* Limine 约定：启动请求（base revision 等）放在这个段里供 bootloader 扫描；
     * KEEP 防止 rustc 默认开启的 --gc-sections 把它回收掉 */
    .limine_requests : {
        KEEP(*(.limine_requests))
    }

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .data : {
        *(.data .data.*)
    }

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .bss : {
        *(COMMON)
        *(.bss .bss.*)
    }

    /DISCARD/ : {
        *(.eh_frame*)
        *(.note .note.*)
    }
}
