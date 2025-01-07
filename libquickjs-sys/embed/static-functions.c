#include "quickjs.h"

// These are static inline functions in quickjs.h so bindgen does not pick
// them up.
// We use define simple wrapper functions to make them available to bindgen,
// and therefore make them usable from Rust.

int JS_ValueGetTag_real(JSValue v)
{
    return JS_VALUE_GET_TAG(v);
}

// used to generate following values:
// JS_NULL
// JS_UNDEFINED
// JS_FALSE
// JS_TRUE
// JS_EXCEPTION
// JS_UNINITIALIZED
JSValue JS_NewSpecialValue_real(uint32_t tag, uint32_t val)
{
    return JS_MKVAL(tag, val);
}

JSValue JS_NewPointer_real(uint32_t tag, void* ptr)
{
    return JS_MKPTR(tag, ptr);
}

JSValue JS_NewFloat64_real(JSContext *ctx, double d)
{
    return JS_NewFloat64(ctx, d);
}

JSValue JS_NewInt32_real(JSContext *ctx, int32_t val)
{
    return JS_NewInt32(ctx, val);
}

JSValue JS_NewBool_real(JSContext *ctx, uint8_t val)
{
    return JS_NewBool(ctx, val);
}

JS_BOOL JS_VALUE_IS_NAN_real(JSValue v)
{
    return JS_VALUE_IS_NAN(v);
}

double JS_VALUE_GET_FLOAT64_real(JSValue v)
{
    return JS_VALUE_GET_FLOAT64(v);
}

int JS_VALUE_GET_INT_real(JSValue v)
{
    return JS_VALUE_GET_INT(v);
}

int JS_VALUE_GET_BOOL_real(JSValue v)
{
    return JS_VALUE_GET_BOOL(v);
}

void* JS_VALUE_GET_PTR_real(JSValue v)
{
    return JS_VALUE_GET_PTR(v);
}

int JS_VALUE_GET_NORM_TAG_real(JSValue v)
{
    return JS_VALUE_GET_NORM_TAG(v);
}

JS_BOOL JS_IsNumber_real(JSValueConst v)
{
    return JS_IsNumber(v);
}

JS_BOOL JS_IsBigInt_real(JSContext *ctx, JSValueConst v)
{
    return JS_IsBigInt(ctx, v);
}

// JS_BOOL JS_IsBigFloat_real(JSValueConst v) {
//     return JS_IsBigFloat(v);
// }

// JS_BOOL JS_IsBigDecimal_real(JSValueConst v) {
//     return JS_IsBigDecimal(v);
// }

JS_BOOL JS_IsBool_real(JSValueConst v)
{
    return JS_IsBool(v);
}

JS_BOOL JS_IsNull_real(JSValueConst v)
{
    return JS_IsNull(v);
}

JS_BOOL JS_IsUndefined_real(JSValueConst v)
{
    return JS_IsUndefined(v);
}

JS_BOOL JS_IsException_real(JSValueConst v)
{
    return JS_IsException(v);
}

JS_BOOL JS_IsUninitialized_real(JSValueConst v)
{
    return JS_IsUninitialized(v);
}

JS_BOOL JS_IsString_real(JSValueConst v)
{
    return JS_IsString(v);
}

JS_BOOL JS_IsSymbol_real(JSValueConst v)
{
    return JS_IsSymbol(v);
}

JS_BOOL JS_IsObject_real(JSValueConst v)
{
    return JS_IsObject(v);
}

int JS_ToUint32_real(JSContext *ctx, uint32_t *pres, JSValueConst val)
{
    return JS_ToUint32(ctx, pres, val);
}

// int JS_SetProperty_real(JSContext *ctx, JSValueConst this_obj, JSAtom prop,
// JSValue val)
// {
//     return JS_SetProperty(ctx, this_obj, prop, val);
// }

JSValue JS_NewCFunction_real(JSContext *ctx,
                             JSCFunction *func,
                             const char *name,
                             int length)
{
    return JS_NewCFunction(ctx, func, name, length);
}

JSValue JS_NewCFunctionMagic_real(JSContext *ctx,
                                  JSCFunctionMagic *func,
                                  const char *name,
                                  int length,
                                  JSCFunctionEnum cproto,
                                  int magic)
{
    return JS_NewCFunctionMagic(ctx, func, name, length, cproto, magic);
}

JS_BOOL JS_IsPromise(JSContext *ctx, JSValue val)
{
    // JS_CLASS_PROMISE == 49
    void *p = JS_GetOpaque(val, 49);

    return p != NULL;
}
