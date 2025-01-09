#define _CRT_SECURE_NO_WARNINGS
#define CONFIG_BIGNUM
#define _GNU_SOURCE
#define WIN32_LEAN_AND_MEAN
#define _WIN32_WINNT 0x0602

#include "quickjs.h"

#ifndef RUSTY_EXTENSION_H
#define RUSTY_EXTENSION_H

#ifdef __cplusplus
extern "C"
{
#endif

  int JS_Ext_ValueGetTag(JSValue v);
  int JS_Ext_GetRefCount(JSValue v);
  JSValue JS_Ext_NewSpecialValue(int32_t tag, uint32_t val);
  JSValue JS_Ext_NewPointer(int32_t tag, void *ptr);
  JSValue JS_Ext_NewFloat64(JSContext *ctx, double d);
  JSValue JS_Ext_NewInt32(JSContext *ctx, int32_t val);
  JSValue JS_Ext_NewBool(JSContext *ctx, uint8_t val);

  JS_BOOL JS_Ext_IsNan(JSValue v);
  double JS_Ext_GetFloat64(JSValue v);
  int JS_Ext_GetInt(JSValue v);
  int JS_Ext_GetBool(JSValue v);
  void *JS_Ext_GetPtr(JSValue v);
  int JS_Ext_GetNormTag(JSValue v);

  JS_BOOL JS_Ext_IsNumber(JSValue v);
  JS_BOOL JS_Ext_IsBigInt(JSContext *ctx, JSValue v);
  JS_BOOL JS_Ext_IsBool(JSValue v);
  JS_BOOL JS_Ext_IsNull(JSValue v);
  JS_BOOL JS_Ext_IsUndefined(JSValue v);
  JS_BOOL JS_Ext_IsException(JSValue v);
  JS_BOOL JS_Ext_IsUninitialized(JSValue v);
  JS_BOOL JS_Ext_IsString(JSValue v);
  JS_BOOL JS_Ext_IsSymbol(JSValue v);
  JS_BOOL JS_Ext_IsObject(JSValue v);

  int JS_Ext_ToUint32(JSContext *ctx, uint32_t *pres, JSValue val);
  JSValue JS_Ext_NewCFunction(JSContext *ctx,
                              JSCFunction *func,
                              const char *name,
                              int length);
  JSValue JS_Ext_NewCFunctionMagic(JSContext *ctx,
                                   JSCFunctionMagic *func,
                                   const char *name,
                                   int length,
                                   JSCFunctionEnum cproto,
                                   int magic);
  JS_BOOL JS_Ext_IsPromise(JSContext *ctx, JSValue val);

  JSValue JS_Ext_PromiseResolve(JSContext *ctx, JSValue value);
  JSValue JS_Ext_PromiseReject(JSContext *ctx, JSValue value);
  JSValue JS_Ext_PromiseAll(JSContext *ctx, JSValue iterable);
  JSValue JS_Ext_PromiseAllSettled(JSContext *ctx, JSValue iterable);
  JSValue JS_Ext_PromiseAny(JSContext *ctx, JSValue iterable);
  JSValue JS_Ext_PromiseRace(JSContext *ctx, JSValue iterable);
  JSValue JS_Ext_PromiseWithResolvers(JSContext *ctx);
  JSValue JS_Ext_PromiseThen(JSContext *ctx, JSValue promise, JSValue on_fulfilled_func);
  JSValue JS_Ext_PromiseThen2(JSContext *ctx, JSValue promise, JSValue on_fulfilled_func, JSValue on_reject_func);
  JSValue JS_Ext_PromiseCatch(JSContext *ctx, JSValue promise, JSValue on_reject_func);
  JSValue JS_Ext_PromiseFinally(JSContext *ctx, JSValue promise, JSValue on_finally_func);

#ifdef __cplusplus
}
#endif
#endif
