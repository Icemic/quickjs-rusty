#include "extensions.h"
#include "./quickjs/quickjs.c"

// These are static inline functions in quickjs.h so bindgen does not pick
// them up.
// We use define simple wrapper functions to make them available to bindgen,
// and therefore make them usable from Rust.

int JS_Ext_ValueGetTag(JSValue v)
{
#ifdef JS_NAN_BOXING
    return JS_VALUE_GET_NORM_TAG(v);
#else
    return JS_VALUE_GET_TAG(v);
#endif
}

int JS_Ext_GetRefCount(JSValue v)
{

    int tag = JS_Ext_ValueGetTag(v);
    if (tag >= JS_TAG_FIRST)
    {
        JSRefCountHeader *ptr = (JSRefCountHeader *)JS_Ext_GetPtr(v);
        return ptr->ref_count;
    }
    else
    {
        return -1;
    }
}

// used to generate following values:
// JS_NULL
// JS_UNDEFINED
// JS_FALSE
// JS_TRUE
// JS_EXCEPTION
// JS_UNINITIALIZED
JSValue JS_Ext_NewSpecialValue(int32_t tag, uint32_t val)
{
    return JS_MKVAL(tag, val);
}

JSValue JS_Ext_NewPointer(int32_t tag, void *ptr)
{
    return JS_MKPTR(tag, ptr);
}

JSValue JS_Ext_NewFloat64(JSContext *ctx, double d)
{
    return JS_NewFloat64(ctx, d);
}

JSValue JS_Ext_NewInt32(JSContext *ctx, int32_t val)
{
    return JS_NewInt32(ctx, val);
}

JSValue JS_Ext_NewBool(JSContext *ctx, uint8_t val)
{
    return JS_NewBool(ctx, val);
}

bool JS_Ext_IsNan(JSValue v)
{
    return JS_VALUE_IS_NAN(v);
}

double JS_Ext_GetFloat64(JSValue v)
{
    return JS_VALUE_GET_FLOAT64(v);
}

int JS_Ext_GetInt(JSValue v)
{
    return JS_VALUE_GET_INT(v);
}

int JS_Ext_GetShortBigInt(JSValue v)
{
    return JS_VALUE_GET_SHORT_BIG_INT(v);
}

int JS_Ext_GetBool(JSValue v)
{
    return JS_VALUE_GET_BOOL(v);
}

void *JS_Ext_GetPtr(JSValue v)
{
    return JS_VALUE_GET_PTR(v);
}

int JS_Ext_GetNormTag(JSValue v)
{
    return JS_VALUE_GET_NORM_TAG(v);
}

bool JS_Ext_IsNumber(JSValue v)
{
    return JS_IsNumber(v);
}

bool JS_Ext_IsBigInt(JSValue v)
{
    return JS_IsBigInt(v);
}

// JS_Ext_BOOL JS_IsBigFloat(JSValue v) {
//     return JS_IsBigFloat(v);
// }

// JS_Ext_BOOL JS_IsBigDecimal(JSValue v) {
//     return JS_IsBigDecimal(v);
// }

bool JS_Ext_IsBool(JSValue v)
{
    return JS_IsBool(v);
}

bool JS_Ext_IsNull(JSValue v)
{
    return JS_IsNull(v);
}

bool JS_Ext_IsUndefined(JSValue v)
{
    return JS_IsUndefined(v);
}

bool JS_Ext_IsException(JSValue v)
{
    return JS_IsException(v);
}

bool JS_Ext_IsUninitialized(JSValue v)
{
    return JS_IsUninitialized(v);
}

bool JS_Ext_IsString(JSValue v)
{
    return JS_IsString(v);
}

bool JS_Ext_IsSymbol(JSValue v)
{
    return JS_IsSymbol(v);
}

bool JS_Ext_IsObject(JSValue v)
{
    return JS_IsObject(v);
}

int JS_Ext_ToUint32(JSContext *ctx, uint32_t *pres, JSValue val)
{
    return JS_ToUint32(ctx, pres, val);
}

JSValue JS_Ext_NewCFunction(JSContext *ctx,
                            JSCFunction *func,
                            const char *name,
                            int length)
{
    return JS_NewCFunction(ctx, func, name, length);
}

JSValue JS_Ext_NewCFunctionMagic(JSContext *ctx,
                                 JSCFunctionMagic *func,
                                 const char *name,
                                 int length,
                                 JSCFunctionEnum cproto,
                                 int magic)
{
    return JS_NewCFunctionMagic(ctx, func, name, length, cproto, magic);
}

bool JS_Ext_IsPromise(JSContext *ctx, JSValue val)
{
    void *p = JS_GetOpaque(val, JS_CLASS_PROMISE);
    return p != NULL;
}

JSValue JS_Ext_PromiseResolve(JSContext *ctx, JSValue value)
{
    return js_promise_resolve(ctx, ctx->promise_ctor, 1, &value, 0);
}

JSValue JS_Ext_PromiseReject(JSContext *ctx, JSValue value)
{
    return js_promise_resolve(ctx, ctx->promise_ctor, 1, &value, 1);
}

JSValue JS_Ext_PromiseAll(JSContext *ctx, JSValue iterable)
{
    return js_promise_all(ctx, ctx->promise_ctor, 1, &iterable, PROMISE_MAGIC_all);
}

JSValue JS_Ext_PromiseAllSettled(JSContext *ctx, JSValue iterable)
{
    return js_promise_all(ctx, ctx->promise_ctor, 1, &iterable, PROMISE_MAGIC_allSettled);
}

JSValue JS_Ext_PromiseAny(JSContext *ctx, JSValue iterable)
{
    return js_promise_all(ctx, ctx->promise_ctor, 1, &iterable, PROMISE_MAGIC_any);
}

JSValue JS_Ext_PromiseRace(JSContext *ctx, JSValue iterable)
{
    return js_promise_race(ctx, ctx->promise_ctor, 1, &iterable);
}

JSValue JS_Ext_PromiseWithResolvers(JSContext *ctx)
{
    return js_promise_withResolvers(ctx, ctx->promise_ctor, 0, NULL);
}

JSValue JS_Ext_PromiseThen(JSContext *ctx, JSValue promise, JSValue on_fulfilled_func)
{
    JSValue argv[1] = {on_fulfilled_func};
    return js_promise_then(ctx, promise, 1, argv);
}

JSValue JS_Ext_PromiseThen2(JSContext *ctx, JSValue promise, JSValue on_fulfilled_func, JSValue on_reject_func)
{
    JSValue argv[2] = {on_fulfilled_func, on_reject_func};
    return js_promise_then(ctx, promise, 2, argv);
}

JSValue JS_Ext_PromiseCatch(JSContext *ctx, JSValue promise, JSValue on_reject_func)
{
    JSValue argv[1] = {on_reject_func};
    return js_promise_catch(ctx, promise, 1, argv);
}

JSValue JS_Ext_PromiseFinally(JSContext *ctx, JSValue promise, JSValue on_finally_func)
{
    JSValue argv[1] = {on_finally_func};
    return js_promise_finally(ctx, promise, 1, argv);
}

JSValue JS_Ext_BigIntToString1(JSContext *ctx, JSValue val, int radix)
{
    return js_bigint_to_string1(ctx, val, radix);
}
