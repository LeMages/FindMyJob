<script setup lang="ts">
// Profile data will come from backend when implemented
</script>

<template>
  <div class="page">
    <h1 class="h1">Mon profil</h1>
    <div class="sub" style="margin-bottom:18px">Ces informations pré-remplissent tes candidatures.</div>

    <!-- Profile header -->
    <div class="phead">
      <div class="pavatar ph" />
      <div class="f1">
        <div class="pname">Alex Martin</div>
        <div class="ptitle">Étudiant·e M2 Data Science — Université Paris-Saclay</div>
        <div class="pcontact">
          <span>alex.martin@etu.fr</span>
          <span>+33 6 12 34 56 78</span>
          <span>Paris, FR</span>
          <span>github.com/alexm</span>
        </div>
      </div>
      <button class="btn">Modifier</button>
    </div>

    <div class="dgrid" style="grid-template-columns:1fr 300px; margin-top:18px">
      <!-- Left: bio, experiences, education -->
      <div>
        <div class="fieldset">
          <div class="fstitle">À PROPOS</div>
          <div class="expd">
            Étudiant·e en M2 Data Science à la recherche d'un stage de fin d'études (6 mois, à partir de janvier 2026).
            Intéressé·e par le NLP et les systèmes de recommandation. Première expérience en équipe data chez Orange.
          </div>
        </div>

        <div class="fieldset">
          <div class="fstitle">EXPÉRIENCES</div>
          <div class="exp">
            <div class="explogo ph" />
            <div class="f1">
              <div class="exprole">Stagiaire Data Science</div>
              <div class="expco">Orange — Paris</div>
              <div class="expmeta">Avr 2024 — Sept 2024 · Stage</div>
              <div class="expd">
                Développement d'un modèle de prédiction de churn (scikit-learn), mise en production via une API Flask, dashboards de suivi.
              </div>
            </div>
          </div>
          <div class="exp" style="border-bottom:none">
            <div class="explogo ph" />
            <div class="f1">
              <div class="exprole">Assistant·e de recherche</div>
              <div class="expco">Laboratoire LISN — Orsay</div>
              <div class="expmeta">Été 2023 · Temps partiel</div>
              <div class="expd">Annotation de corpus et fine-tuning de modèles de classification de textes.</div>
            </div>
          </div>
        </div>

        <div class="fieldset">
          <div class="fstitle">FORMATION</div>
          <div class="exp">
            <div class="explogo ph" />
            <div class="f1">
              <div class="exprole">M2 Data Science</div>
              <div class="expco">Université Paris-Saclay</div>
              <div class="expmeta">2024 — 2025</div>
            </div>
          </div>
          <div class="exp" style="border-bottom:none">
            <div class="explogo ph" />
            <div class="f1">
              <div class="exprole">Licence Maths-Informatique</div>
              <div class="expco">Sorbonne Université</div>
              <div class="expmeta">2021 — 2024</div>
            </div>
          </div>
        </div>
      </div>

      <!-- Right: CV, skills, languages -->
      <div>
        <div class="fieldset">
          <div class="fstitle">CV</div>
          <div class="doc">
            <div class="docicon ph" />
            <div class="f1">
              <div class="crole">CV_AlexMartin_2025.pdf</div>
              <div class="cmeta">Mis à jour 02 juin</div>
            </div>
          </div>
          <div class="newses">+ Remplacer le CV</div>
        </div>

        <div class="fieldset">
          <div class="fstitle">COMPÉTENCES</div>
          <div class="chips">
            <span class="chip">Python</span>
            <span class="chip">PyTorch</span>
            <span class="chip">SQL</span>
            <span class="chip">scikit-learn</span>
            <span class="chip">Pandas</span>
            <span class="chip">NLP</span>
            <span class="chip">Docker</span>
            <span class="chip">Git</span>
            <span class="chip">Spark</span>
            <span class="chip">AWS</span>
          </div>
        </div>

        <div class="fieldset">
          <div class="fstitle">LANGUES</div>
          <div class="setrow" style="padding:8px 0">
            <span class="setl">Français</span><span class="cmeta">Natif</span>
          </div>
          <div class="setrow" style="padding:8px 0">
            <span class="setl">Anglais</span><span class="cmeta">C1</span>
          </div>
          <div class="setrow" style="padding:8px 0;border-bottom:none">
            <span class="setl">Espagnol</span><span class="cmeta">B1</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
