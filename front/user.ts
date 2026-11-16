import "./styles/index.css";
import type { Post } from "./post.ts";
import { makePostShared } from "./post.ts";

const logInButton = document.querySelector<HTMLButtonElement>("#log-in")!;
const logOutButton = document.querySelector<HTMLButtonElement>("#log-out")!;

const feed = document.querySelector<HTMLElement>("#feed")!;

let currentUserId: number | null = null;
let user: string | null = null;

logOutButton.addEventListener("click", async () => {
  try {
    await fetch("/api/user/logout", {
      method: "POST",
    });

    window.location.reload();
  } catch {
    window.location.reload();
  }
});

const makePost = (post: Post) => makePostShared(post, currentUserId, showPosts);

async function showPosts() {
  feed.setAttribute("aria-busy", "true");

  try {
    const posts = await fetch(`/api/post/user/${user}`, {
      credentials: "include",
    });

    if (!posts.ok) {
      console.error(`Could not load posts (${posts.status})`);
      showMessage("Could not load posts.");
      return;
    }

    feed.replaceChildren();

    const list: Post[] = await posts.json();
    list.forEach((p) => feed.append(makePost(p)));

    const endMessage = document.createElement("p");
    endMessage.className = "feed-empty";
    endMessage.textContent = "No more posts.";
    feed.append(endMessage);
  } finally {
    feed.removeAttribute("aria-busy");
  }
}

function showMessage(message: string) {
  const p = document.createElement("p");
  p.className = "feed-empty";
  p.textContent = message;
  feed.replaceChildren(p);
}

async function init() {
  const params = new URLSearchParams(window.location.search);

  const paramsUser = params.get("user");

  if (paramsUser) {
    user = paramsUser;
  } else {
    showMessage("User not found");
  }

  try {
    const res = await fetch("/api/user/get", { credentials: "include" });
    if (res.ok) {
      const me = await res.json();
      currentUserId = me.id;
      logOutButton.hidden = false;
    }
  } catch {
    logInButton.hidden = false;
    logInButton.addEventListener("click", () => {
      window.location.href = "/login.html";
    });
  }
  if (user) await showPosts();
}
init();
