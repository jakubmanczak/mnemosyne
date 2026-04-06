document.addEventListener("DOMContentLoaded", () => {
  const container = document.querySelector("[quotelines]");
  const template = document.querySelector("[quotelinetemplate]");
  const addButton = document.querySelector("[addlinebtn]");

  addButton.addEventListener("click", () => {
    const clone = template.content.cloneNode(true);
    container.appendChild(clone);
  });

  container.addEventListener("click", (e) => {
    const rmBtn = e.target.closest("[rmlinebtn]");
    if (rmBtn && !rmBtn.disabled) {
      const line = rmBtn.closest("[quoteline]");
      if (line && container.querySelectorAll("[quoteline]").length > 1) {
        line.remove();
      }
    }
  });
});
