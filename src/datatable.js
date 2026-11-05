// The datatable widget pack. The server renders each state, so the table
// works with JavaScript off. This file only:
// - keeps the select-all box, its mixed state and the count in sync,
// - stops a bulk submit with no row and says why,
// - asks for a confirm action in the table's <dialog>.
// It uses native checkboxes: Tab moves focus and Space toggles a box.
(function () {
  'use strict';

  var ROOT = '[data-autumn-datatable]';

  function own(root, selector) {
    return Array.prototype.filter.call(root.querySelectorAll(selector), function (el) {
      return el.closest(ROOT) === root;
    });
  }

  function rowBoxes(root) {
    return own(root, '[data-autumn-datatable-row]').filter(function (box) { return !box.disabled; });
  }

  function setStatus(root, text) {
    var status = own(root, '[data-autumn-datatable-status]')[0];
    // A live region reads each change. Write only a new text.
    if (status && status.textContent !== text) status.textContent = text;
  }

  // Read the row boxes. Write the select-all state and the count.
  function sync(root) {
    var boxes = rowBoxes(root);
    var count = boxes.filter(function (box) { return box.checked; }).length;
    var all = own(root, '[data-autumn-datatable-all]')[0];
    if (all) {
      var mixed = count > 0 && count < boxes.length;
      all.checked = boxes.length > 0 && count === boxes.length;
      // The native state and aria-checked change together.
      all.indeterminate = mixed;
      if (mixed) all.setAttribute('aria-checked', 'mixed');
      else all.removeAttribute('aria-checked');
    }
    setStatus(root, count + ' of ' + boxes.length + ' selected');
    return count;
  }

  var ready = new WeakSet();
  function setUp(root) {
    if (!ready.has(root)) {
      ready.add(root);
      root.setAttribute('data-autumn-datatable-ready', 'true');
    }
    sync(root);
  }

  // The target can be a root, hold roots, or be inside a root (a swap of
  // the rows only).
  document.addEventListener('autumn:init', function (e) {
    var target = e.target;
    if (!target || !target.querySelectorAll) return;
    var outer = target.closest && target.closest(ROOT);
    if (outer) setUp(outer);
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

  function stop(e) {
    e.preventDefault();
    // htmx (hx-boost) listens on the form. Stop it too.
    e.stopImmediatePropagation();
  }

  // The form of the confirmed submit. It is set only during requestSubmit.
  var confirmed = null;
  var returnFocus = new WeakMap();

  // Capture: this runs before a listener on the form, for example htmx.
  document.addEventListener('submit', function (e) {
    var form = e.target;
    if (!form || !form.matches || !form.matches('[data-autumn-datatable-form]')) return;
    var root = form.closest(ROOT);
    if (!root || confirmed === form) return;
    if (sync(root) === 0) {
      stop(e);
      setStatus(root, form.getAttribute('data-autumn-datatable-none') || '');
      return;
    }
    var select = own(root, 'select[name="action"]')[0];
    var option = select && select.options[select.selectedIndex];
    var text = option && option.getAttribute('data-autumn-datatable-confirm');
    var dialog = document.getElementById(root.id + '-confirm');
    if (!text || !dialog || typeof dialog.showModal !== 'function') return;
    stop(e);
    var message = dialog.querySelector('[data-autumn-datatable-confirm-text]');
    if (message) message.textContent = text;
    returnFocus.set(dialog, e.submitter || document.activeElement);
    dialog.showModal();
  }, true);

  document.addEventListener('click', function (e) {
    var button = e.target && e.target.closest
      && e.target.closest('[data-autumn-datatable-ok],[data-autumn-datatable-cancel]');
    if (!button) return;
    var dialog = button.closest('dialog');
    var root = dialog && dialog.closest(ROOT);
    var form = root && own(root, '[data-autumn-datatable-form]')[0];
    if (!dialog || !form) return;
    var ok = button.matches('[data-autumn-datatable-ok]');
    var submitter = returnFocus.get(dialog);
    dialog.close();
    if (!ok) return;
    // The server reads `confirmed=1`. Without it, a confirm action asks
    // again on the server.
    var flag = document.createElement('input');
    flag.type = 'hidden';
    flag.name = 'confirmed';
    flag.value = '1';
    form.appendChild(flag);
    confirmed = form;
    try {
      // Both read the form data before they return.
      if (typeof form.requestSubmit === 'function') {
        form.requestSubmit(submitter && submitter.form === form ? submitter : undefined);
      } else {
        form.submit();
      }
    } finally {
      confirmed = null;
      flag.remove();
    }
  });

  // Escape or Cancel: give focus back to the submit button.
  // `close` does not bubble, so listen in the capture phase.
  document.addEventListener('close', function (e) {
    var dialog = e.target;
    if (!dialog || !dialog.matches || !dialog.matches('dialog.autumn-datatable-confirm')) return;
    var back = returnFocus.get(dialog);
    returnFocus.delete(dialog);
    if (back && back.isConnected && typeof back.focus === 'function') back.focus();
  }, true);
})();
