
#define	SHIM_FUNC(sym)	void sym(void) { asm volatile ("ud2a" : : : ); }

unsigned long __stack_chk_guard;

SHIM_FUNC(__chk_fail)
SHIM_FUNC(__gets_chk)
SHIM_FUNC(__memcpy_chk)
SHIM_FUNC(__memmove_chk)
SHIM_FUNC(__memset_chk)
SHIM_FUNC(__snprintf_chk)
SHIM_FUNC(__sprintf_chk)
SHIM_FUNC(__stack_chk_fail)
SHIM_FUNC(__stpcpy_chk)
SHIM_FUNC(__strcat_chk)
SHIM_FUNC(__strcpy_chk)
SHIM_FUNC(__strncat_chk)
SHIM_FUNC(__strncpy_chk)
SHIM_FUNC(__vsnprintf_chk)
SHIM_FUNC(__vsprintf_chk)
