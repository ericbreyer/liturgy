<template>
  <div class="about-view">
    <div class="hero-section">
      <div class="hero-content">
        <h1 class="hero-title">Liturgical Calendar</h1>
        <p class="hero-subtitle">Liturgical calendars across different traditions and forms</p>
      </div>
    </div>

    <div class="content-section">
      <div class="section">
        <h2>About This Application</h2>
        <p>
          This application provides a comprehensive comparison tool for liturgical calendars,
          allowing users to explore and compare feast days, commemorations, and liturgical seasons
          across different forms of the Roman Rite and other traditions.
        </p>
      </div>

      <div class="section">
        <h2>Features</h2>
        <div class="features-grid">
          <div class="feature-card">
            <h3>Daily View</h3>
            <p>View liturgical information for any specific day across selected calendars</p>
          </div>

          <div class="feature-card">
            <h3>Week & Month Views</h3>
            <p>Browse liturgical calendars by week or month for broader planning</p>
          </div>

          <div class="feature-card">
            <h3>Search</h3>
            <p>Search for specific feasts and commemorations across all calendars</p>
          </div>

          <div class="feature-card">
            <h3>Detailed Comparison</h3>
            <p>
              Advanced comparison showing differences, transfers, and rank changes between calendars
            </p>
          </div>
        </div>
      </div>

      <div class="section">
        <h2>Supported Calendars</h2>
        <div class="calendars-list">
          <div class="calendar-item">
            <h3>Ordinary Form (OF)</h3>
            <p>The Roman Calendar as reformed after Vatican II (1969)</p>
          </div>

          <div class="calendar-item">
            <h3>Extraordinary Form (EF)</h3>
            <p>The Traditional Roman Calendar (1962 Missal)</p>
          </div>

          <div class="calendar-item">
            <h3>OF with US Extensions</h3>
            <p>Ordinary Form calendar with United States proper celebrations</p>
          </div>
        </div>
      </div>

      <div class="section">
        <h2>How to Use</h2>
        <ol class="steps-list">
          <li>
            <strong>Select Calendars:</strong> Choose which liturgical calendars you want to compare
            using the calendar selector
          </li>
          <li>
            <strong>Navigate Dates:</strong> Use the date selector to browse different days, weeks,
            or months
          </li>
          <li>
            <strong>Compare & Analyze:</strong> View liturgical information side by side, search for
            specific feasts, or use the detailed comparison view
          </li>
        </ol>
      </div>

      <div class="section">
        <h2>Technical Information</h2>
        <p>
          This application is built with modern web technologies to provide fast, responsive access
          to liturgical calendar data. The backend is written in Rust using the Axum framework,
          while the frontend uses Vue 3 with TypeScript for a smooth user experience.
        </p>

        <ul class="tech-list">
          <li><strong>Backend:</strong> Rust + Axum + TOML data format</li>
          <li><strong>Frontend:</strong> Vue 3 + TypeScript + Vite</li>
          <li>
            <strong>Features:</strong> Custom fuzzy search, responsive design, real-time comparison
          </li>
        </ul>
      </div>

      <div class="section">
        <h2>Data Sources</h2>
        <p>
          The liturgical calendar data is compiled from official sources and traditional liturgical
          references, ensuring accuracy and authenticity in the representation of feast days,
          commemorations, and liturgical seasons.
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
@import '../styles/liturgical.css';

.about-view {
  /* width: var(--layout-fixed-width); */
  max-width: 100vw; /* Fallback for very small screens */
  margin: 0 auto;
  padding: var(--layout-padding);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  box-sizing: border-box;
}

.hero-section {
  text-align: center;
  padding: 40px 20px;
  margin-bottom: 40px;
  border-bottom: 1px solid var(--border-primary);
}

.hero-title {
  font-size: 2.5rem;
  font-weight: 300;
  margin-bottom: 16px;
  color: var(--accent-color);
}

.hero-subtitle {
  font-size: 1.1rem;
  color: var(--text-secondary);
  max-width: 500px;
  margin: 0 auto;
  line-height: 1.5;
}

.content-section {
  line-height: 1.6;
}

.section {
  margin-bottom: 40px;
}

.section h2 {
  color: var(--text-primary);
  font-size: 1.5rem;
  font-weight: 400;
  margin-bottom: 16px;
  border-bottom: 2px solid var(--accent-color);
  padding-bottom: 8px;
}

.section p {
  color: var(--text-secondary);
  margin-bottom: 16px;
}

.features-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
  margin-top: 20px;
}

.feature-card {
  padding: 20px;
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  background: var(--surface-primary);
}

.feature-card h3 {
  color: var(--text-primary);
  margin-bottom: 12px;
  font-size: 1.1rem;
  font-weight: 500;
}

.feature-card p {
  color: var(--text-secondary);
  font-size: 0.95rem;
  margin: 0;
}

.calendars-list {
  margin-top: 20px;
}

.calendar-item {
  margin-bottom: 20px;
  padding: 16px;
  border-left: 3px solid var(--accent-color);
  background: var(--surface-primary);
}

.calendar-item h3 {
  color: var(--text-primary);
  margin-bottom: 8px;
  font-size: 1.1rem;
  font-weight: 500;
}

.calendar-item p {
  color: var(--text-secondary);
  margin: 0;
  font-size: 0.95rem;
}

.steps-list {
  margin-top: 20px;
  padding-left: 20px;
}

.steps-list li {
  margin-bottom: 16px;
  color: var(--text-secondary);
}

.steps-list strong {
  color: var(--accent-color);
}

.tech-list {
  margin-top: 20px;
  padding-left: 20px;
}

.tech-list li {
  margin-bottom: 8px;
  color: var(--text-secondary);
}

.tech-list strong {
  color: var(--text-primary);
}

@media (max-width: 768px) {
  .about-view {
    padding: 16px;
  }

  .hero-title {
    font-size: 2rem;
  }

  .features-grid {
    grid-template-columns: 1fr;
  }

  .section h2 {
    font-size: 1.3rem;
  }
}
</style>
