<script setup lang="ts">
/**
 * Aide repliable sur le balisage des consignes d'une référence (`.typ`),
 * affichée à l'endroit de l'import. Le format complet est décrit dans
 * `docs/reference-markup.md` ; ce composant n'en reprend que l'essentiel.
 */
</script>

<template>
  <details class="mt-[10px] text-[12px] text-neutral-600">
    <summary class="cursor-pointer select-none text-neutral-500 hover:text-neutral-700">Balisage des consignes</summary>
    <div class="mt-2 flex flex-col gap-[6px] leading-[1.45]">
      <p>Les consignes s'écrivent en commentaires <code class="font-mono">//</code> dans le fichier .typ ; elles n'apparaissent pas dans le PDF.</p>
      <ul class="list-disc pl-5 flex flex-col gap-[3px]">
        <li><code class="font-mono">// [CONSIGNES]</code> puis des lignes <code class="font-mono">//</code> : consignes générales, jusqu'à la première ligne qui n'est pas un commentaire.</li>
        <li><code class="font-mono">// [FIXE] consigne</code> : le texte qui suit est recopié mot pour mot (traduit fidèlement si le document est généré dans une autre langue).</li>
        <li><code class="font-mono">// [VARIABLE] consigne</code> : le texte qui suit est réécrit d'après l'offre.</li>
        <li><code class="font-mono">// [ADAPTABLE] consigne</code> : le texte qui suit est réécrit d'après votre profil, sans rien inventer.</li>
      </ul>
      <p>Une zone va jusqu'à la balise suivante ou jusqu'à la ligne vide. Les balises s'appliquent au texte rédigé : objet et paragraphes de la lettre, titre et résumé du CV. Un texte rédigé sans balise est traité comme <code class="font-mono">[VARIABLE]</code>.</p>
      <p>La balise doit ouvrir le commentaire : <code class="font-mono">// --- Corps --- [FIXE]</code> est refusé.</p>
    </div>
  </details>
</template>
