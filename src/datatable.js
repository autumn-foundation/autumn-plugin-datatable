// The datatable widget pack. The server renders each state, so the table
// works with JavaScript off. This file only:
// - keeps the select-all box, its mixed state and the count in sync,
// - stops a bulk submit with no row and says why,
// - asks for a confirm action in the table's <dialog>.
// It uses native checkboxes: Tab moves focus and Space toggles a box.
(function () {
  'use strict';

  var ROOT = '[data-autumn-datatable]';

  function rowBoxes(root) {
    return Array.prototype.filter.call(
      root.querySelectorAll('[data-autumn-datatable-row]'),
      function (box) { return !box.disabled && box.closest(ROOT) === root; }
    );
  }

  function allBox(root) {
    var box = root.querySelector('[data-autumn-datatable-all]');
    return box && box.closest(ROOT) === root ? box : null;
  }

  function setStatus(root, text) {
    var status = root.querySelector('[data-autumn-datatable-status]');
    if (status) status.textContent = text;
  }

  // Read the row boxes. Write the select-all state and the count.
  function sync(root) {
    var boxes = rowBoxes(root);
    var count = boxes.filter(function (box) { return box.checked; }).length;
    var all = allBox(root);
    if (all) {
      var mixed = count > 0 && count < boxes.length;
      all.checked = boxes.length > 0 && count === boxes.length;
      all.indeterminate = mixed;
      if (mixed) all.setAttribute('aria-checked', 'mixed');
      else all.removeAttribute('aria-checked');
    }
    setStatus(root, count + ' of ' + boxes.length + ' selected');
    return count;
  }

  var ready = new WeakSet();
  function setUp(root) {
    if (ready.has(root)) return;
    ready.add(root);
    sync(root);
    root.setAttribute('data-autumn-datatable-ready', 'true');
  }

  document.addEventListener('autumn:init', function (e) {
    var target = e.target;
    if (!target || !target.querySelectorAll) return;
    if (target.matches && target.matches(ROOT)) setUp(target);
    target.querySelectorAll(ROOT).forEach(setUp);
  });

  document.addEventListener('change', function (e) {
    var box = e.target;
    if (!box || !box.closest) return;
    var root = box.closest(ROOT);
    if (!root) return;
    if (box.matches('[data-autumn-datatable-all]')) {
      rowBoxes(root).forEach(function (row) { row.checked = box.checked; });
      sync(root);
    } else if (box.matches('[data-autumn-datatable-row]')) {
      sync(root);
    }
  });

  // A confirmed submit runs one time without the dialog.
  var confirmed = new WeakSet();

  document.addEventListener('submit', function (e) {
    var form = e.target;
    if (!form || !form.matches || !form.matches('[data-autumn-datatable-form]')) return;
    var root = form.closest(ROOT);
    if (!root) return;
    if (confirmed.has(form)) {
      confirmed.delete(form);
      return;
    }
    if (sync(root) === 0 && !(allBox(root) || {}).checked) {
      e.preventDefault();
      setStatus(root, form.getAttribute('data-autumn-datatable-none') || '');
      return;
    }
    var select = form.querySelector('select[name="action"]');
    var option = select && select.options[select.selectedIndex];
    var text = option && option.getAttribute('data-autumn-datatable-confirm');
    var dialog = document.getElementById(root.id + '-confirm');
    if (!text || !dialog || typeof dialog.showModal !== 'function') return;
    e.preventDefault();
    var message = dialog.querySelector('[data-autumn-datatable-confirm-text]');
    if (message) message.textContent = text;
    dialog.returnFocus = e.submitter || document.activeElement;
    dialog.showModal();
  });

  document.addEventListener('click', function (e) {
    var button = e.target && e.target.closest
      && e.target.closest('[data-autumn-datatable-ok],[data-autumn-datatable-cancel]');
    if (!button) return;
    var dialog = button.closest('dialog');
    var root = dialog && dialog.closest(ROOT);
    var form = root && root.querySelector('[data-autumn-datatable-form]');
    if (!dialog || !form) return;
    var ok = button.matches('[data-autumn-datatable-ok]');
    var submitter = dialog.returnFocus;
    dialog.close();
    if (!ok) return;
    confirmed.add(form);
    if (typeof form.requestSubmit === 'function') {
      form.requestSubmit(submitter && submitter.form === form ? submitter : undefined);
    } else {
      form.submit();
    }
  });

  // Escape or Cancel: give focus back to the submit button.
  document.addEventListener('close', function (e) {
    var dialog = e.target;
    if (!dialog || !dialog.matches || !dialog.matches('dialog.autumn-datatable-confirm')) return;
    var back = dialog.returnFocus;
    dialog.returnFocus = null;
    if (back && back.isConnected && typeof back.focus === 'function') back.focus();
  }, true);
})();
