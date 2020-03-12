
#define	SHIM_FUNC(sym)	void sym(void) { asm volatile ("ud2a" : : : ); }


#ifdef VER_7_0_0
#define VER_4_8_0

/* Functions for GCC_7.0.0 */
SHIM_FUNC(__divmodti4)

#endif /* VER_7_0_0 */

#ifdef VER_4_8_0
#define VER_4_7_0

/* Functions for GCC_4.8.0 */
unsigned long __cpu_model;
SHIM_FUNC(__cpu_indicator_init)

#endif /* VER_4_8_0 */


#ifdef VER_4_7_0
#define VER_4_5_0

/* Functions for GCC_4.7.0 */
SHIM_FUNC(__clrsbdi2)
SHIM_FUNC(__clrsbti2)

#endif /* VER_4_7_0 */

#ifdef VER_4_5_0
#define VER_4_3_0

/* Functions for GCC_4.5.0 */
SHIM_FUNC(__addtf3)
SHIM_FUNC(__divtc3)
SHIM_FUNC(__divtf3)
SHIM_FUNC(__eqtf2)
SHIM_FUNC(__extenddftf2)
SHIM_FUNC(__extendsftf2)
SHIM_FUNC(__extendxftf2)
SHIM_FUNC(__fixtfdi)
SHIM_FUNC(__fixtfsi)
SHIM_FUNC(__fixtfti)
SHIM_FUNC(__fixunstfdi)
SHIM_FUNC(__fixunstfsi)
SHIM_FUNC(__fixunstfti)
SHIM_FUNC(__floatditf)
SHIM_FUNC(__floatsitf)
SHIM_FUNC(__floattitf)
SHIM_FUNC(__floatunditf)
SHIM_FUNC(__floatunsitf)
SHIM_FUNC(__floatuntitf)
SHIM_FUNC(__getf2)
SHIM_FUNC(__gttf2)
SHIM_FUNC(__letf2)
SHIM_FUNC(__lttf2)
SHIM_FUNC(__multc3)
SHIM_FUNC(__multf3)
SHIM_FUNC(__negtf2)
SHIM_FUNC(__netf2)
SHIM_FUNC(__powitf2)
SHIM_FUNC(__subtf3)
SHIM_FUNC(__trunctfdf2)
SHIM_FUNC(__trunctfsf2)
SHIM_FUNC(__trunctfxf2)
SHIM_FUNC(__unordtf2)

#endif /* VER_4_5_0 */

#ifdef VER_4_3_0
#define VER_4_2_0

/* Functions for GCC_4.3.0 */
SHIM_FUNC(__bswapdi2)
SHIM_FUNC(__bswapsi2)
SHIM_FUNC(__emutls_get_address)
SHIM_FUNC(__emutls_register_common)

#endif /* VER_4_3_0 */

#ifdef VER_4_2_0
#define VER_4_0_0

/* Functions for GCC_4.2.0 */
SHIM_FUNC(__floatuntidf)
SHIM_FUNC(__floatuntisf)
SHIM_FUNC(__floatuntixf)
SHIM_FUNC(_Unwind_GetIPInfo)

#endif /* VER_4_2_0 */

#ifdef VER_4_0_0
#define VER_3_4_4

/* Functions for GCC_4.0.0 */
SHIM_FUNC(__divdc3)
SHIM_FUNC(__divsc3)
SHIM_FUNC(__divxc3)
SHIM_FUNC(__muldc3)
SHIM_FUNC(__mulsc3)
SHIM_FUNC(__mulxc3)
SHIM_FUNC(__powidf2)
SHIM_FUNC(__powisf2)
SHIM_FUNC(__powixf2)

#endif /* VER_4_0_0 */

#ifdef VER_3_4_4
#define VER_3_4_2

/* Functions for GCC_3.4.4 */
SHIM_FUNC(__absvti2)
SHIM_FUNC(__addvti3)
SHIM_FUNC(__mulvti3)
SHIM_FUNC(__negvti2)
SHIM_FUNC(__subvti3)

#endif /* VER_3_4_4 */

#ifdef VER_3_4_2
#define VER_3_4_0

/* Functions for GCC_3.4.2 */
SHIM_FUNC(__enable_execute_stack)

#endif /* VER_3_4_2 */

#ifdef VER_3_4_0
#define VER_3_3_1

/* Functions for GCC_3.4 */
SHIM_FUNC(__clzdi2)
SHIM_FUNC(__clzti2)
SHIM_FUNC(__ctzdi2)
SHIM_FUNC(__ctzti2)
SHIM_FUNC(__paritydi2)
SHIM_FUNC(__parityti2)
SHIM_FUNC(__popcountdi2)
SHIM_FUNC(__popcountti2)

#endif /* VER_3_4_0 */

#ifdef VER_3_3_1
#define VER_3_3_0

/* Functions for GCC_3.3.1 */
SHIM_FUNC(__gcc_personality_v0)

#endif /* VER_3_3_1 */

#ifdef VER_3_3_0
#define VER_3_0_0

/* Functions for GCC_3.3 */
SHIM_FUNC(_Unwind_Backtrace)
SHIM_FUNC(_Unwind_FindEnclosingFunction)
SHIM_FUNC(_Unwind_GetCFA)
SHIM_FUNC(_Unwind_Resume_or_Rethrow)

#endif /* VER_3_3_0 */

#ifdef VER_3_0_0

/* Functions for GCC_3.0 */
SHIM_FUNC(__absvdi2)
SHIM_FUNC(__absvsi2)
SHIM_FUNC(__addvdi3)
SHIM_FUNC(__addvsi3)
SHIM_FUNC(__ashlti3)
SHIM_FUNC(__ashrti3)
SHIM_FUNC(__clear_cache)
SHIM_FUNC(__cmpti2)
SHIM_FUNC(__deregister_frame)
SHIM_FUNC(__deregister_frame_info)
SHIM_FUNC(__deregister_frame_info_bases)
SHIM_FUNC(__divti3)
SHIM_FUNC(__ffsdi2)
SHIM_FUNC(__ffsti2)
SHIM_FUNC(__fixdfti)
SHIM_FUNC(__fixsfti)
SHIM_FUNC(__fixunsdfdi)
SHIM_FUNC(__fixunsdfti)
SHIM_FUNC(__fixunssfdi)
SHIM_FUNC(__fixunssfti)
SHIM_FUNC(__fixunsxfdi)
SHIM_FUNC(__fixunsxfti)
SHIM_FUNC(__fixxfti)
SHIM_FUNC(__floattidf)
SHIM_FUNC(__floattisf)
SHIM_FUNC(__floattixf)
SHIM_FUNC(__lshrti3)
SHIM_FUNC(__modti3)
SHIM_FUNC(__multi3)
SHIM_FUNC(__mulvdi3)
SHIM_FUNC(__mulvsi3)
SHIM_FUNC(__negti2)
SHIM_FUNC(__negvdi2)
SHIM_FUNC(__negvsi2)
SHIM_FUNC(__register_frame)
SHIM_FUNC(__register_frame_info)
SHIM_FUNC(__register_frame_info_bases)
SHIM_FUNC(__register_frame_info_table)
SHIM_FUNC(__register_frame_info_table_bases)
SHIM_FUNC(__register_frame_table)
SHIM_FUNC(__subvdi3)
SHIM_FUNC(__subvsi3)
SHIM_FUNC(__ucmpti2)
SHIM_FUNC(__udivmodti4)
SHIM_FUNC(__udivti3)
SHIM_FUNC(__umodti3)
SHIM_FUNC(_Unwind_DeleteException)
SHIM_FUNC(_Unwind_Find_FDE)
SHIM_FUNC(_Unwind_ForcedUnwind)
SHIM_FUNC(_Unwind_GetDataRelBase)
SHIM_FUNC(_Unwind_GetGR)
SHIM_FUNC(_Unwind_GetIP)
SHIM_FUNC(_Unwind_GetLanguageSpecificData)
SHIM_FUNC(_Unwind_GetRegionStart)
SHIM_FUNC(_Unwind_GetTextRelBase)
SHIM_FUNC(_Unwind_RaiseException)
SHIM_FUNC(_Unwind_Resume)
SHIM_FUNC(_Unwind_SetGR)
SHIM_FUNC(_Unwind_SetIP)

#endif /* VER_3_0_0 */
