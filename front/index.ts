import "./styles/index.css";

const authAction = document.querySelector<HTMLButtonElement>("#auth-action")!;
const feed = document.querySelector<HTMLElement>("#feed")!;

async function init() {
  let loggedIn = false;

  try {
    const res = await fetch("/api/user/get", { credentials: "include" });
    loggedIn = res.ok;
  } catch {
    // Not logged in if u can't connect (401).
  }

  authAction.hidden = false;

  if (loggedIn) {
    authAction.textContent = "Post";
    // TODO: open a message/post composer thingy.
  } else {
    authAction.textContent = "Log in";
    authAction.addEventListener("click", () => {
      window.location.href = "/login.html";
    });
  }

  // then also later actual feed loading and stuff and all the logic associated with that
  await new Promise((resolve) => setTimeout(resolve, 400));

  feed.replaceChildren();
  feed.removeAttribute("aria-busy");

  const empty = document.createElement("p");
  empty.className = "feed__empty";
  empty.textContent = "No posts yet.";
  feed.append(empty);
}

init();
