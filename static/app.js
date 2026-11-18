let allCards = [];
let lastDataSnapshot = "";

const clpFormatter = new Intl.NumberFormat("es-CL", {
  style: "currency",
  currency: "CLP",
  maximumFractionDigits: 0,
});

const RARITY_LABELS = {
  m: "Mítica",
  r: "Rara",
  u: "Infrecuente",
  c: "Común",
  p: "Promo",
  s: "Especial",
  t: "Ficha",
  l: "Tierra",
};

const RARITY_RANK = {
  m: 5,
  r: 4,
  p: 3.5,
  s: 3.5,
  u: 3,
  c: 2,
  l: 1,
  t: 0,
};

function escapeHtml(value) {
  if (value === null || value === undefined) return "";
  return String(value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

function parseColors(rawColors) {
  if (Array.isArray(rawColors)) return rawColors;
  try {
    const parsed = JSON.parse(rawColors || "[]");
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function setSyncStatus(connected) {
  const dot = document.getElementById("sync-dot");
  const text = document.getElementById("sync-text");
  if (connected) {
    dot.className = "w-2 h-2 rounded-full bg-emerald-400 animate-pulse";
    text.textContent = "En vivo";
  } else {
    dot.className = "w-2 h-2 rounded-full bg-rose-500";
    text.textContent = "Sin conexión";
  }
}

async function fetchCards(forceRender = false) {
  try {
    const resp = await fetch("/api/cards");
    if (!resp.ok) throw new Error("Error del servidor");

    const cards = await resp.json();
    const snapshot = JSON.stringify(cards);
    setSyncStatus(true);

    if (forceRender || snapshot !== lastDataSnapshot) {
      allCards = cards;
      lastDataSnapshot = snapshot;
      renderDashboard();
    }
  } catch (err) {
    console.error("Error al obtener cartas:", err);
    setSyncStatus(false);
  }
}

function updateStats(cards) {
  let totalQty = 0;
  let totalClp = 0;
  let totalUsd = 0;

  for (const card of cards) {
    totalQty += card.quantity;
    if (card.price_clp !== null) totalClp += card.price_clp * card.quantity;
    if (card.price_usd !== null) totalUsd += card.price_usd * card.quantity;
  }

  document.getElementById("stat-total-qty").textContent = totalQty.toLocaleString("es-CL");
  document.getElementById("stat-unique-cards").textContent = cards.length.toLocaleString("es-CL");
  document.getElementById("stat-total-clp").textContent = clpFormatter.format(totalClp);
  document.getElementById("stat-total-usd").textContent = `$${totalUsd.toFixed(2)}`;
}

function getRarityRank(rarity) {
  const key = (rarity || "").toLowerCase();
  return RARITY_RANK[key] ?? -1;
}

function getFilteredAndSortedCards() {
  const query = document.getElementById("search-input").value.trim().toLowerCase();
  const rarityFilter = document.getElementById("rarity-filter").value;
  const colorFilter = document.getElementById("color-filter").value;
  const binFilter = document.getElementById("bin-filter").value;
  const sortBy = document.getElementById("sort-select").value;
  const foilOnly = document.getElementById("foil-only-checkbox").checked;

  const filtered = allCards.filter((card) => {
    if (foilOnly && !card.is_foil) return false;

    if (rarityFilter) {
      const r = (card.rarity || "").toLowerCase();
      if (rarityFilter === "p") {
        if (r !== "p" && r !== "s") return false;
      } else if (r !== rarityFilter) {
        return false;
      }
    }

    if (colorFilter) {
      const colors = parseColors(card.colors);
      if (colorFilter === "C") {
        if (colors.length > 0) return false;
      } else if (!colors.includes(colorFilter)) {
        return false;
      }
    }

    if (binFilter) {
      if (String(card.bin_number ?? "") !== binFilter) return false;
    }

    if (query) {
      const haystack = [
        card.name,
        card.name_en,
        card.name_es,
        card.set_code,
        card.collector_number,
      ]
        .filter(Boolean)
        .join(" ")
        .toLowerCase();

      if (!haystack.includes(query)) return false;
    }

    return true;
  });

  filtered.sort((a, b) => {
    switch (sortBy) {
      case "price_desc":
        return (b.price_clp ?? -1) - (a.price_clp ?? -1);
      case "price_asc":
        return (a.price_clp ?? Number.MAX_SAFE_INTEGER) - (b.price_clp ?? Number.MAX_SAFE_INTEGER);
      case "rarity_desc": {
        const diff = getRarityRank(b.rarity) - getRarityRank(a.rarity);
        return diff !== 0 ? diff : (b.price_clp ?? -1) - (a.price_clp ?? -1);
      }
      case "rarity_asc": {
        const diff = getRarityRank(a.rarity) - getRarityRank(b.rarity);
        return diff !== 0 ? diff : (a.price_clp ?? Number.MAX_SAFE_INTEGER) - (b.price_clp ?? Number.MAX_SAFE_INTEGER);
      }
      case "qty_desc":
        return b.quantity - a.quantity;
      case "name_asc":
        return a.name.localeCompare(b.name, "es");
      case "recent":
      default:
        return 0;
    }
  });

  return filtered;
}

function renderDashboard() {
  updateStats(allCards);

  const cards = getFilteredAndSortedCards();
  const grid = document.getElementById("cards-grid");
  const emptyState = document.getElementById("empty-state");
  const resultsCount = document.getElementById("results-count");

  resultsCount.textContent = `Mostrando ${cards.length} de ${allCards.length} ediciones`;

  if (cards.length === 0) {
    grid.innerHTML = "";
    emptyState.classList.remove("hidden");
    return;
  }

  emptyState.classList.add("hidden");

  grid.innerHTML = cards
    .map((card) => {
      const rarityCode = (card.rarity || "c").toLowerCase();
      const rarityLabel = RARITY_LABELS[rarityCode] || rarityCode.toUpperCase();
      const priceClpText =
        card.price_clp !== null ? clpFormatter.format(card.price_clp) : "Sin CLP";
      const priceUsdText =
        card.price_usd !== null ? `$${card.price_usd.toFixed(2)} USD` : "Sin USD";
      const binText =
        card.bin_number !== null ? `Bandeja #${card.bin_number}` : "Sin bandeja";

      const secondaryName =
        card.name_en && card.name_en.toLowerCase() !== card.name.toLowerCase()
          ? `<p class="text-xs text-slate-400 truncate" title="${escapeHtml(card.name_en)}">${escapeHtml(card.name_en)}</p>`
          : "";

      const foilBadge = card.is_foil
        ? `<span class="foil-badge px-2 py-0.5 rounded text-[11px] font-bold uppercase tracking-wider text-slate-950 shadow">★ Foil</span>`
        : "";

      const imageHtml = card.image_url
        ? `<img
            src="${escapeHtml(card.image_url)}"
            alt="${escapeHtml(card.name)}"
            loading="lazy"
            onerror="this.style.display='none'; this.nextElementSibling.classList.remove('hidden');"
            class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
          />
          <div class="hidden w-full h-full flex flex-col items-center justify-center p-4 text-center bg-slate-950 text-slate-400">
            <span class="text-xs uppercase tracking-widest text-slate-500">${escapeHtml(card.set_code)} #${escapeHtml(card.collector_number)}</span>
            <span class="font-semibold text-slate-200 mt-1">${escapeHtml(card.name)}</span>
          </div>`
        : `<div class="w-full h-full flex flex-col items-center justify-center p-4 text-center bg-slate-950 text-slate-400">
            <span class="text-xs uppercase tracking-widest text-slate-500">${escapeHtml(card.set_code)} #${escapeHtml(card.collector_number)}</span>
            <span class="font-semibold text-slate-200 mt-1">${escapeHtml(card.name)}</span>
          </div>`;

      return `
        <article class="group bg-slate-900 border border-slate-800 ${card.is_foil ? "foil-card" : ""} rounded-xl overflow-hidden flex flex-col justify-between transition hover:border-slate-600">
          <div>
            <!-- Arte de la Carta -->
            <div class="relative aspect-[488/680] bg-slate-950 overflow-hidden">
              ${imageHtml}

              <!-- Etiquetas Superiores: Bandeja y Foil -->
              <div class="absolute top-2.5 left-2.5 right-2.5 flex items-center justify-between gap-1.5">
                <span class="px-2.5 py-0.5 rounded-md bg-slate-950/90 border border-slate-700 text-xs font-semibold text-indigo-300 backdrop-blur">
                  ${escapeHtml(binText)}
                </span>
                ${foilBadge}
              </div>

              <!-- Cantidad en Stock -->
              <div class="absolute bottom-2.5 right-2.5 px-2.5 py-1 rounded-lg bg-slate-950/90 border border-slate-700 text-xs font-bold text-white backdrop-blur">
                Stock: ${card.quantity}
              </div>
            </div>

            <!-- Datos de la Carta -->
            <div class="p-3.5 space-y-2">
              <div>
                <h2 class="font-semibold text-white text-sm leading-snug truncate" title="${escapeHtml(card.name)}">
                  ${escapeHtml(card.name)}
                </h2>
                ${secondaryName}
              </div>

              <div class="flex items-center flex-wrap gap-1.5 text-[11px]">
                <span class="px-2 py-0.5 rounded bg-slate-800 text-slate-300 font-mono uppercase">
                  ${escapeHtml(card.set_code)} #${escapeHtml(card.collector_number)}
                </span>
                <span class="px-2 py-0.5 rounded bg-slate-800 text-slate-300 uppercase">
                  ${escapeHtml(card.lang)}
                </span>
                <span class="px-2 py-0.5 rounded rarity-${escapeHtml(rarityCode)} font-medium">
                  ${escapeHtml(rarityLabel)}
                </span>
              </div>
            </div>
          </div>

          <!-- Precio y Controles de Stock -->
          <div class="px-3.5 pb-3.5 pt-2 border-t border-slate-800/80 flex items-center justify-between gap-2">
            <div>
              <div class="text-base font-bold text-emerald-400 leading-tight">${priceClpText}</div>
              <div class="text-[11px] text-slate-400">${priceUsdText}</div>
            </div>

            <div class="flex items-center gap-1">
              <button
                onclick="adjustStock(${card.id}, -1)"
                title="Vender / Restar 1"
                class="w-7 h-7 rounded-md bg-slate-800 hover:bg-rose-600/80 text-slate-200 hover:text-white font-bold text-sm transition"
              >
                -
              </button>
              <button
                onclick="adjustStock(${card.id}, 1)"
                title="Agregar 1"
                class="w-7 h-7 rounded-md bg-slate-800 hover:bg-emerald-600/80 text-slate-200 hover:text-white font-bold text-sm transition"
              >
                +
              </button>
              <button
                onclick="deleteCard(${card.id}, '${escapeHtml(card.name)}')"
                title="Eliminar registro"
                class="w-7 h-7 rounded-md bg-slate-800 hover:bg-slate-700 text-slate-400 hover:text-rose-400 text-xs transition ml-0.5"
              >
                ✕
              </button>
            </div>
          </div>
        </article>
      `;
    })
    .join("");
}

async function adjustStock(id, delta) {
  try {
    const resp = await fetch(`/api/cards/${id}`, {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ delta }),
    });
    if (resp.ok) {
      await fetchCards(true);
    }
  } catch (err) {
    console.error("Error al ajustar stock:", err);
  }
}

async function deleteCard(id, name) {
  if (!confirm(`¿Eliminar todas las copias de "${name}" del inventario?`)) return;
  try {
    const resp = await fetch(`/api/cards/${id}`, { method: "DELETE" });
    if (resp.ok) {
      await fetchCards(true);
    }
  } catch (err) {
    console.error("Error al eliminar carta:", err);
  }
}

async function refreshPrices() {
  const btn = document.getElementById("refresh-prices-btn");
  const label = document.getElementById("refresh-label");
  const icon = document.getElementById("refresh-icon");

  btn.disabled = true;
  label.textContent = "Actualizando...";
  icon.classList.add("animate-spin");

  try {
    const resp = await fetch("/api/prices/refresh", { method: "POST" });
    if (resp.ok) {
      await fetchCards(true);
    }
  } catch (err) {
    console.error("Error al actualizar precios:", err);
  } finally {
    btn.disabled = false;
    label.textContent = "Actualizar Precios";
    icon.classList.remove("animate-spin");
  }
}

function openResetModal() {
  document.getElementById("reset-modal").classList.remove("hidden");
}

function closeResetModal() {
  document.getElementById("reset-modal").classList.add("hidden");
}

async function confirmResetDatabase() {
  const btn = document.getElementById("confirm-reset-btn");
  btn.disabled = true;
  btn.textContent = "Borrando...";

  try {
    const resp = await fetch("/api/cards", { method: "DELETE" });
    if (resp.ok) {
      lastDataSnapshot = "";
      closeResetModal();
      await fetchCards(true);
    }
  } catch (err) {
    console.error("Error al reiniciar la base de datos:", err);
  } finally {
    btn.disabled = false;
    btn.textContent = "Sí, borrar todo";
  }
}

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    closeResetModal();
  }
});

// Carga inicial + actualización cada 3 segundos
fetchCards(true);
setInterval(() => fetchCards(false), 3000);