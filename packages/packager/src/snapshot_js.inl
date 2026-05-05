(function(){
  try {
    var root = document.documentElement;
    function walk(el, depth) {
      if (!el || el.nodeType !== 1) return '';
      var tag = (el.tagName||'').toLowerCase();
      var out = '  '.repeat(depth);
      if (tag === '#text') { out += JSON.stringify(el.textContent); }
      else {
        out += '<' + tag;
        if (el.id) out += ' id="' + el.id + '"';
        if (el.className && typeof el.className==='string') out += ' class="' + el.className + '"';
        var role = el.getAttribute('role');
        if (role) out += ' role="' + role + '"';
        var href = el.getAttribute('href');
        if (href) out += ' href="' + href + '"';
        var type = el.getAttribute('type')||el.type;
        if (type) out += ' type="' + type + '"';
        var name = el.getAttribute('name')||el.name;
        if (name) out += ' name="' + name + '"';
        var value = el.value;
        if (value!==undefined&&value!==null&&value!=='') out += ' value="' + value + '"';
        var placeholder = el.placeholder;
        if (placeholder) out += ' placeholder="' + placeholder + '"';
        var ariaLabel = el.getAttribute('aria-label');
        if (ariaLabel) out += ' aria-label="' + ariaLabel + '"';
        var text = '';
        for (var c=0;c<el.childNodes.length;c++) {
          var cn=el.childNodes[c];
          if (cn.nodeType===3) { text+=cn.textContent.trim(); }
        }
        if (text) out += ' "' + text.substring(0,80) + '"';
        out += '>';
        for (var i=0;i<el.children.length;i++) { out+='\n'+walk(el.children[i],depth+1); }
      }
      return out;
    }
    return walk(root,0);
  } catch(e) { return JSON.stringify({error:e.message}); }
})()
