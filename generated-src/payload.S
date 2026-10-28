.section .rodata.stress_payload,"a",%progbits
.balign 16
.global stress_payload_start
.hidden stress_payload_start
.type stress_payload_start,%object
stress_payload_start:
.incbin "build/payload.bin"
.global stress_payload_end
.hidden stress_payload_end
stress_payload_end:
.size stress_payload_start, stress_payload_end-stress_payload_start
