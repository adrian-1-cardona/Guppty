/* Progressive enhancements for the docs page. No dependencies.
   Everything here is optional: the page reads fine with JavaScript off. */

(function () {
  "use strict";

  /* ----------------------------------------------------------------------
     Heading anchors — hover a heading to reveal a link to it.
     ---------------------------------------------------------------------- */
  document.querySelectorAll("h2[id], h3[id]").forEach(function (heading) {
    var anchor = document.createElement("a");
    anchor.className = "anchor";
    anchor.href = "#" + heading.id;
    anchor.textContent = "#";
    anchor.setAttribute("aria-label", "Link to this section");
    heading.appendChild(anchor);
  });

  /* ----------------------------------------------------------------------
     Copy buttons on code blocks.
     ---------------------------------------------------------------------- */
  document.querySelectorAll(".content pre").forEach(function (pre) {
    var wrapper = document.createElement("div");
    wrapper.className = "snippet";
    pre.parentNode.insertBefore(wrapper, pre);
    wrapper.appendChild(pre);

    var button = document.createElement("button");
    button.type = "button";
    button.className = "copy";
    button.textContent = "Copy";
    button.setAttribute("aria-label", "Copy this example");
    wrapper.appendChild(button);

    button.addEventListener("click", function () {
      var text = pre.innerText;
      var done = function () {
        button.textContent = "Copied";
        button.dataset.done = "true";
        window.setTimeout(function () {
          button.textContent = "Copy";
          delete button.dataset.done;
        }, 1400);
      };

      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(done, function () {
          button.textContent = "Failed";
        });
        return;
      }

      // Fallback for browsers without the async clipboard API.
      var field = document.createElement("textarea");
      field.value = text;
      field.setAttribute("readonly", "");
      field.style.position = "absolute";
      field.style.left = "-9999px";
      document.body.appendChild(field);
      field.select();
      try {
        document.execCommand("copy");
        done();
      } catch (error) {
        button.textContent = "Failed";
      }
      document.body.removeChild(field);
    });
  });

  /* ----------------------------------------------------------------------
     Highlight the sidebar entry for the section currently on screen.
     ---------------------------------------------------------------------- */
  var links = Array.prototype.slice.call(
    document.querySelectorAll("#toc a[href^='#']")
  );
  if (!links.length || !("IntersectionObserver" in window)) {
    return;
  }

  var linkFor = {};
  var targets = [];

  links.forEach(function (link) {
    var id = link.getAttribute("href").slice(1);
    var target = document.getElementById(id);
    if (!target) {
      return;
    }
    linkFor[id] = link;
    targets.push(target);
  });

  var visible = new Set();

  var setCurrent = function (id) {
    links.forEach(function (link) {
      link.classList.remove("is-current");
    });
    if (linkFor[id]) {
      linkFor[id].classList.add("is-current");
    }
  };

  var observer = new IntersectionObserver(
    function (entries) {
      entries.forEach(function (entry) {
        if (entry.isIntersecting) {
          visible.add(entry.target.id);
        } else {
          visible.delete(entry.target.id);
        }
      });

      // Pick the first section, in document order, that is on screen.
      for (var i = 0; i < targets.length; i += 1) {
        if (visible.has(targets[i].id)) {
          setCurrent(targets[i].id);
          return;
        }
      }
    },
    { rootMargin: "-72px 0px -70% 0px", threshold: 0 }
  );

  targets.forEach(function (target) {
    observer.observe(target);
  });
})();
