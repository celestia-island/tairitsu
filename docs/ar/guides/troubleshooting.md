# دليل استكشاف الأخطاء وإصلاحها

المشاكل الشائعة والحلول عند العمل مع Tairitsu browser-glue ونموذج المكونات.

## أخطاء البناء

### هدف wasm32-wasip2 غير موجود

**الخطأ:**
```
error: can't find crate for `std`
  |
  = note: the `wasm32-wasip2` target may not be installed
```

**الحل:**
```bash
rustup target add wasm32-wasip2
```

### عدم تطابق إصدار wit-bindgen

**الخطأ:**
```
error: failed to select a version for `wit-bindgen`
```

**الحل:**
تأكد من أن إصدار `wit-bindgen` يتطابق في `Cargo.toml`:
```toml
[dependencies]
wit-bindgen = { version = "0.33", features = ["realloc"] }
```

### أخطاء تجميع TypeScript

**الخطأ:**
```
error TS2307: Cannot find module './domGlue' or its corresponding type declarations.
```

**الحل:**
أعد توليد الـ glue وأعد البناء:
```bash
cd packages/browser-glue
npm run build
```

## أخطاء وقت التشغيل

### استيرادات المضيف مفقودة

**الخطأ:**
```
Error: Component import "tairitsu-browser:full/document" was not satisfied
```

**الحل:**
1. تأكد من تكوين خريطة الاستيراد:
```html
<script type="importmap">
{
  "imports": {
    "tairitsu-browser:full/": "./browser-glue/"
  }
}
</script>
```

2. تحقق من وجود ملفات browser-glue في مجلد الإخراج.

### فشل تهيئة المكون

**الخطأ:**
```
Error: Component instantiation failed: undefined import
```

**الحل:**
تحقق من أن جميع استيرادات WIT المطلوبة لها تطبيقات مقابلة في browser-glue.

### أخطاء تحويل jco

**الخطأ:**
```
Error: Failed to transpile component
```

**الحل:**
1. تأكد من تثبيت jco:
```bash
npm install -g @bytecodealliance/jco
```

2. تحقق من صحة مكون WASM:
```bash
wasm-tools print component.wasm
```

## تقنيات التصحيح

### تفعيل سجلات التصحيح

في وحدة تحكم المتصفح:
```javascript
localStorage.setItem('debug', 'tairitsu:*');
```

### فحص روابط WIT

عرض الروابط المُولّدة:
```bash
cat packages/web/src/wit_platform.rs | head -100
```

### أدوات المطور في المتصفح

1. افتح أدوات المطور (F12)
2. تحقق من وحدة التحكم للأخطاء
3. تبويب الشبكة لعمليات تحميل الوحدات الفاشلة
4. تبويب المصادر للتصحيح

### التحقق من صحة المكون

```bash
# التحقق من بنية المكون
wasm-tools validate component.wasm

# طباعة محتويات المكون
wasm-tools print component.wasm
```

## المشاكل الشائعة

### المقابض غير موجودة

**العرض:** إرجاع `null` من عمليات DOM

**السبب:** تم جمع المقابض كقمامة أو لم يتم تسجيلها

**الحل:** تأكد من أن العناصر تبقى مرجعية في JavaScript

### الأحداث لا تعمل

**العرض:** معالجات الأحداث لا تُستدعى

**السبب:** عدم تطابق معرف المستمع أو نوع الحدث غير صحيح

**الحل:** تحقق من أن `addEventListener` يُرجع معرف مستمع صالح

### تسرب الذاكرة

**العرض:** زيادة استخدام الذاكرة مع الوقت

**السبب:** المقابض لم يتم تحريرها بعد الاستخدام

**الحل:** استدعِ `dropHandle()` عند الانتهاء من الكائنات

## مشاكل الأداء

### بطء تحميل المكون

**الحلول:**
1. استخدم بناء الإصدار: `cargo build --release`
2. فعّل LTO في `Cargo.toml`:
```toml
[profile.release]
lto = true
opt-level = 'z'
```

### زمن انتقال عالٍ للأحداث

**الحلول:**
1. تجنب العمليات المتزامنة في المعالجات
2. استخدم `requestAnimationFrame` للتحديثات المرئية
3. قلل من الأحداث السريعة المتتالية (Debounce)

## الحصول على المساعدة

1. تحقق من المشاكل الموجودة: https://github.com/anomalyco/opencode/issues
2. راجع الوثائق في مجلد `docs/`
3. افحص الكود المثال في `examples/website/`
