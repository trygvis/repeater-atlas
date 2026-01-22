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
  style.textContent = "#repeater-map { height: 100%; width: 100%; }";
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

function initializeRepeaterAtlas() {
  const leafletBase = "/static/vendor/leaflet/images/";
  L.Icon.Default.mergeOptions({
    iconUrl: `${leafletBase}marker-icon.png`,
    iconRetinaUrl: `${leafletBase}marker-icon-2x.png`,
    shadowUrl: `${leafletBase}marker-shadow.png`,
  });
}
