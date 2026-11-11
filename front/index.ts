import "./styles/index.css";

const authAction = document.querySelector<HTMLButtonElement>("#auth-action")!;
const logOutButton = document.querySelector<HTMLButtonElement>("#log-out")!;
const feed = document.querySelector<HTMLElement>("#feed")!;

// shape the api **should** send
type Post = {
  id: number;
  text: string;
  image: string;
};

// fake posts for dev server: http://localhost:5173/?mock=1
// this stuff will be modified or removed whenever the API is APIng
const mockPosts: Post[] = [
  {
    id: 1,
    text: "jooo hienohienohienohienohienohienohienohienohienohienohienohienohienohienohienohienohienohienohienohienohieno teksti",
    image: "/front/assets/hero.png",
  },
  { id: 2, text: "toinen postaus", image: "/front/assets/hero.png" },
  {
    id: 3,
    text: "kolmas eeppinen testi-postaus",
    image: "/front/assets/hero.png",
  },
];

logOutButton.addEventListener("click", async () => {
  try {
    await fetch("/api/user/logout", {
      method: "POST",
    });

    window.location.href = "/";
    return;
  } catch {
    window.location.href = "/";
    return;
  }
});

async function loadPosts(): Promise<Post[]> {
  if (import.meta.env.DEV && location.search.includes("mock")) {
    return mockPosts;
  }
  const res = await fetch("/api/posts", { credentials: "include" });

  if (!res.ok) {
    throw new Error(`could not load posts (${res.status})`);
  }
  const data = await res.json();
  return Array.isArray(data) ? data : (data.posts ?? []);
}

// build one post "card"
function makePost(post: Post) {
  const card = document.createElement("article");
  card.className = "post";

  const img = document.createElement("img");
  img.className = "post-image";
  img.src = post.image;
  img.loading = "lazy";
  img.onerror = () => card.classList.add("post-image-missing");

  const text = document.createElement("p");
  text.className = "post-text";
  text.textContent = post.text;

  card.append(text, img);
  return card;
}

async function showPosts() {
  try {
    const posts = await loadPosts();

    feed.replaceChildren();

    let count = 0;
    for (const post of posts) {
      if (!post.image) {
        // images are required, so a post without one is essentially "broken data". IMO, this is a cat social media, you HAVE to post cat pics
        console.warn("skipping post without image:", post.id);
        continue;
      }
      feed.append(makePost(post));
      count++;
    }

    if (count === 0) {
      showMessage("No posts yet.");
    }
  } catch (error) {
    console.error(error);
    showMessage("Could not load posts.");
  }

  feed.removeAttribute("aria-busy");
}

function showMessage(message: string) {
  const p = document.createElement("p");
  p.className = "feed-empty";
  p.textContent = message;
  feed.replaceChildren(p);
}

async function init() {
  let loggedIn = false;

  try {
    const res = await fetch("/api/user/get", { credentials: "include" });
    loggedIn = res.ok;
  } catch {
    // Not logged in if you can't connect (401).
  }

  authAction.hidden = false;

  if (loggedIn) {
    authAction.textContent = "Post";
    logOutButton.hidden = false;
    // TODO: open a message/post composer thingy.
  } else {
    authAction.textContent = "Log in";
    authAction.addEventListener("click", () => {
      window.location.href = "/login.html";
    });
  }

  await showPosts();
}
init();
