function buildLeafletMap(host) {
  const root = host.attachShadow({mode: "open"});
  const stylesheet = document.createElement("link");
  stylesheet.rel = "stylesheet";
  stylesheet.href = "/static/vendor/leaflet/leaflet.css";
  root.append(stylesheet);
  const clusterStylesheet = document.createElement("link");
  clusterStylesheet.rel = "stylesheet";
  clusterStylesheet.href = "/static/vendor/leaflet.markercluster/MarkerCluster.css";
  root.append(clusterStylesheet);
  const clusterDefaultStylesheet = document.createElement("link");
  clusterDefaultStylesheet.rel = "stylesheet";
  clusterDefaultStylesheet.href = "/static/vendor/leaflet.markercluster/MarkerCluster.Default.css";
  root.append(clusterDefaultStylesheet);
  const style = document.createElement("style");
  style.textContent = `
    #repeater-map { height: 100%; width: 100%; }
    .marker-cluster {
      color: var(--pico-color)
    }
    .ra-marker-label.leaflet-tooltip {
      border: 1px solid rgba(0, 0, 0, 0.25);
      border-radius: 6px;
      background: rgba(255, 255, 255, 0.9);
      color: #111;
      font-size: 12px;
      font-weight: 600;
      letter-spacing: 0.02em;
      padding: 2px 6px;
      box-shadow: 0 1px 6px rgba(0, 0, 0, 0.2);
    }
  `;
  root.append(style);
  const mapElement = document.createElement("div");
  mapElement.id = "repeater-map";
  root.append(mapElement);

  const map = L.map(mapElement);

  const layer = L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
    attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
  });

  layer.addTo(map);

  return map;
}

function addCallSignLabel(marker, callSign) {
  marker.bindTooltip(callSign, {
    permanent: true,
    direction: "right",
    offset: [6, 0],
    opacity: 0.9,
    className: "ra-marker-label",
  });
}

function initializeRepeaterAtlas() {
  const leafletBase = "/static/vendor/leaflet/images/";
  L.Icon.Default.mergeOptions({
    iconUrl: `${leafletBase}marker-icon.png`,
    iconRetinaUrl: `${leafletBase}marker-icon-2x.png`,
    shadowUrl: `${leafletBase}marker-shadow.png`,
  });
}
