<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let searchQuery = $state("a");
  let suggestions = $state<string[]>([]);
  let showSuggestions = $state(true);
  let searchForm: HTMLFormElement | null = null;
  let searchInputClass = $state("");
  $effect(() => {
    searchInputClass =
      showSuggestions && suggestions.length > 0
        ? "border-x-1 border-t-1 border-b-0 rounded-t-lg"
        : "border-1 rounded-lg";
  });
  /*
   * This function handles the input event of the search input field.
   */
  async function handleSearchInput() {
    if (searchQuery.trim() === "") {
      suggestions = [];
      showSuggestions = false;
      return;
    }

    // TODO: Fetch suggestions from a Backend
    const sampleSuggestions = [
      "Tauri",
      "Svelte",
      "Vite",
      "TypeScript",
      "Rust",
      "JavaScript",
      "TailwindCSS",
      "Frontend",
      "Backend",
      "WebAssembly",
      "Web App",
      "Progressive Web App",
      "Cross-Platform",
      "Native App",
      "Electron",
      "React",
      "Desktop App",
    ];
    suggestions = sampleSuggestions.filter((item) =>
      item.toLowerCase().includes(searchQuery.toLowerCase()),
    );
    showSuggestions = true;
    console.log("Suggestions:", suggestions);
  }

  /*
   * This function handles the selection of a suggestion from the list.
   */
  function selectSuggestion(suggestion: string) {
    searchQuery = suggestion;
    showSuggestions = false;
    if (searchForm) {
      searchForm.dispatchEvent(new Event("submit", { cancelable: true }));
    }
  }

  // Handle search form submission
  function handleSearch(event: Event) {
    event.preventDefault();
    console.log("Searching for:", searchQuery);
    // Here you would typically process the search
    showSuggestions = false;
  }
  handleSearchInput();

  // async function greet(event: Event) {
  //   event.preventDefault();
  //   // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  //   greetMsg = await invoke("greet", { name });
  // }
</script>

<main class={["w-full", "h-full", "bg-transparent"]}>
  <div class={["w-full"]}>
    <form
      class={["w-full", "relative"]}
      onsubmit={handleSearch}
      bind:this={searchForm}
    >
      <div
        data-tauri-drag-region
        class={[
          "w-full",
          "display-flex",
          "relative",
          "bg-(--color-bg)",
          "border-solid",
          "border-(--color-line)",
          "pr-3",
          searchInputClass,
        ]}
      >
        <input
          type="text"
          class={[
            "w-[calc(100%-10px)]",
            "pl-3",
            "pr-3",
            "m-3",
            "text-2xl",
            "ease-linear",
            "bg-(--color-bg)",
            "placeholder:text-(--color-line)",
            "text-(--color-text)",
            "shadow-(--shadow-base)",
            "outline-none",
          ]}
          placeholder="Application name..."
          bind:value={searchQuery}
          oninput={handleSearchInput}
        />
        <button
          type="submit"
          class={[
            "absolute",
            "right-[15px]",
            "top-[50%]",
            "transform-[translateY(-50%)]",
            "background-none",
            "border-none",
            "padding-[8px]",
            "cursor-pointer",
            "display-flex",
            "align-items-center",
            "justify-content-center",
          ]}
          aria-label="Search"
        >
          <svg
            viewBox="0 0 24 24"
            class={["w-[52px]", "h-[52px]", "text-(--color-text)"]}
          >
            <path
              fill="currentColor"
              d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"
            />
          </svg>
        </button>
      </div>
      {#if showSuggestions && suggestions.length > 0}
        <ul
          class={[
            "w-full",
            "z-10",
            "px-[1px]",
            "border-x-1",
            "border-b-1",
            "border-solid",
            "border-(--color-line)",
            "rounded-b-lg",
            "overflow-auto",
            "max-h-[15em]",
          ]}
        >
          {#each suggestions as suggestion}
            <button
              type="button"
              class={[
                "w-full",
                "text-2xl",
                "text-left",
                "pl-6",
                "py-2",
                "bg-(--color-bg-light)",
                "hover:bg-(--color-bg-lightx2)",
                "text-(--color-text)",
                "last:rounded-b-lg",
              ]}
              onmousedown={() => selectSuggestion(suggestion)}
            >
              {suggestion}
            </button>
          {/each}
        </ul>
      {/if}
    </form>
  </div>
</main>

<style lang="postcss">
  @reference "tailwindcss";
</style>
