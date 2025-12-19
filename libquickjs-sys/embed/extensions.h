#include "./quickjs/quickjs.h"

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

  bool JS_Ext_IsNan(JSValue v);
  double JS_Ext_GetFloat64(JSValue v);
  int JS_Ext_GetInt(JSValue v);
  int JS_Ext_GetShortBigInt(JSValue v);
  int JS_Ext_GetBool(JSValue v);
  void *JS_Ext_GetPtr(JSValue v);
  int JS_Ext_GetNormTag(JSValue v);

  bool JS_Ext_IsNumber(JSValue v);
  bool JS_Ext_IsBigInt(JSValue v);
  bool JS_Ext_IsBool(JSValue v);
  bool JS_Ext_IsNull(JSValue v);
  bool JS_Ext_IsUndefined(JSValue v);
  bool JS_Ext_IsException(JSValue v);
  bool JS_Ext_IsUninitialized(JSValue v);
  bool JS_Ext_IsString(JSValue v);
  bool JS_Ext_IsSymbol(JSValue v);
  bool JS_Ext_IsObject(JSValue v);

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
  bool JS_Ext_IsPromise(JSContext *ctx, JSValue val);

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
  JSValue JS_Ext_BigIntToString1(JSContext *ctx, JSValue val, int radix);

  typedef JSValue JSIteratorNextFunc(JSContext *ctx, JSValueConst this_val,
                                     int argc, JSValueConst *argv, int *pdone, int magic);
  typedef JSValue JSGetterFunc(JSContext *ctx, JSValueConst this_val);
  typedef JSValue JSSetterFunc(JSContext *ctx, JSValueConst this_val, JSValueConst val);
  typedef JSValue JSGetterMagicFunc(JSContext *ctx, JSValueConst this_val, int magic);
  typedef JSValue JSSetterMagicFunc(JSContext *ctx, JSValueConst this_val, JSValueConst val, int magic);

  JSCFunctionListEntry JS_Ext_CFunc_Def(const char *name, uint8_t length, JSCFunction *func1);
  JSCFunctionListEntry JS_Ext_CFunc_Def_C_W_E(const char *name, uint8_t length, JSCFunction *func1);
  JSCFunctionListEntry JS_Ext_CFunc_Def2(const char *name, uint8_t length, JSCFunction *func1, uint8_t prop_flags);
  JSCFunctionListEntry JS_Ext_CFunc_Magic_Def(const char *name, uint8_t length, JSCFunctionMagic *func1, int16_t magic);
  JSCFunctionListEntry JS_Ext_Iterator_Next_Def(const char *name, uint8_t length, JSIteratorNextFunc *func1, int16_t magic);
  JSCFunctionListEntry JS_Ext_CGetSet_Def(const char *name, JSGetterFunc *fgetter, JSSetterFunc *fsetter);
  JSCFunctionListEntry JS_Ext_CGetSet_Def2(const char *name, JSGetterFunc *fgetter, JSSetterFunc *fsetter, uint8_t prop_flags);
  JSCFunctionListEntry JS_Ext_CGetSet_Magic_Def(const char *name, JSGetterMagicFunc *fgetter, JSSetterMagicFunc *fsetter, int16_t magic);
  JSCFunctionListEntry JS_Ext_Prop_String_Def(const char *name, const char *cstr, uint8_t prop_flags);
  JSCFunctionListEntry JS_Ext_Prop_Int32_Def(const char *name, int32_t val, uint8_t prop_flags);
  JSCFunctionListEntry JS_Ext_Prop_Int64_Def(const char *name, int64_t val, uint8_t prop_flags);
  JSCFunctionListEntry JS_Ext_Prop_Double_Def(const char *name, double val, uint8_t prop_flags);
  JSCFunctionListEntry JS_Ext_Prop_U2D_Def(const char *name, uint64_t val, uint8_t prop_flags);
  JSCFunctionListEntry JS_Ext_Prop_Undefined_Def(const char *name, uint8_t prop_flags);
  JSCFunctionListEntry JS_Ext_Object_Def(const char *name, const JSCFunctionListEntry *tab, int len, uint8_t prop_flags);
  JSCFunctionListEntry JS_Ext_Alias_Def(const char *name, const char *from);
  JSCFunctionListEntry JS_Ext_Alias_Base_Def(const char *name, const char *from, int base);

#ifdef __cplusplus
}
#endif
#endif
