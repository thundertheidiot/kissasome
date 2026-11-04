import "./styles/login.css" 

const form = document.querySelector<HTMLFormElement>("#login-form")!;
const username = document.querySelector<HTMLInputElement>("#username")!;
const password = document.querySelector<HTMLInputElement>("#password")!;
const reveal = document.querySelector<HTMLButtonElement>("#reveal")!;
const submit = document.querySelector<HTMLButtonElement>(".login__submit")!;
const error = document.querySelector<HTMLParagraphElement>("#error")!;

// show & hide passwd
reveal.addEventListener("click", () => {
  const hidden = password.type === "password";
  password.type = hidden ? "text" : "password";
  reveal.textContent = hidden ? "Hide" : "Show";
  reveal.setAttribute("aria-label", hidden ? "Hide password" : "Show password");
});

// login button press
form.addEventListener("submit", (e) => {
  e.preventDefault();
  error.hidden = true;
  // TODO: login jutut
  
});

// a func for errors to be shown
function showError(message: string) {
  error.textContent = message;
  error.hidden = false;
}
