import express from "express";
import dotenv from "dotenv";

dotenv.config();

console.log("SERVER VERSION: FULL_HTTP_PLUGIN_V2");

const app = express();
app.use(express.json({ limit: "1mb" }));

/** -------------------------
 * Helpers (general)
 * ------------------------*/
function colId(c) {
  // Luzmo stuurt soms column_id of id
  return c?.column_id ?? c?.id;
}

function makeReqId() {
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`;
}

function normalizeValue(v) {
  // Luzmo sometimes sends numbers wrapped in arrays: [19.95]
  if (Array.isArray(v) && v.length === 1) return v[0];
  return v;
}

function resolveColumnId(obj) {
  return obj?.column_id ?? obj?.id ?? obj?.column ?? obj?.field ?? null;
}

const round2 = (x) => (typeof x === "number" ? Math.round(x * 100) / 100 : x);

/** Date bucketing for group-by */
function bucketDateISO(iso, level) {
  if (!level) return iso;

  const d = new Date(iso);
  const y = d.getUTCFullYear();
  const m = String(d.getUTCMonth() + 1).padStart(2, "0");
  const day = String(d.getUTCDate()).padStart(2, "0");

  if (level === "year") return `${y}-01-01T00:00:00.000Z`;
  if (level === "quarter") {
    const q = Math.floor(d.getUTCMonth() / 3) + 1;
    const qm = String((q - 1) * 3 + 1).padStart(2, "0");
    return `${y}-${qm}-01T00:00:00.000Z`;
  }
  if (level === "month") return `${y}-${m}-01T00:00:00.000Z`;
  if (level === "day") return `${y}-${m}-${day}T00:00:00.000Z`;

  return iso;
}

function getCell(row, id, colIndex) {
  const idx = colIndex[id];
  return idx === undefined ? undefined : row[idx];
}

function makeGroupKey(values) {
  return JSON.stringify(values);
}

function toTime(v) {
  v = normalizeValue(v);
  if (typeof v !== "string") return null;
  const t = Date.parse(v);
  return Number.isNaN(t) ? null : t;
}

/** Raw mode: select only requested columns */
function projectRows(rows, columns, colIndex) {
  const ids = (columns ?? []).map(colId).filter(Boolean);

  // If Luzmo didn't specify columns, return full rows
  if (!ids.length) return rows;

  const idxs = ids.map((id) => colIndex[id]).filter((i) => i !== undefined);
  return rows.map((r) => idxs.map((i) => r[i]));
}

/** -------------------------
 * Aggregation engine
 * ------------------------*/
function aggregateRows(rows, groupByCols, measures, colIndex) {
  // groupByCols: original column objects without aggregation
  // measures: [{ column_id, aggregation }]
  const groups = new Map();

  for (const row of rows) {
    const groupVals = groupByCols.map((c) => {
      const id = colId(c);
      const v = getCell(row, id, colIndex);

      const isDateCol = id === "date" || c?.type === "datetime";
      if (isDateCol && typeof v === "string" && c?.level) {
        return bucketDateISO(v, c.level);
      }
      return v;
    });

    const key = makeGroupKey(groupVals);

    if (!groups.has(key)) {
      const initState = measures.map((m) => {
        if (m.aggregation === "count") return 0;
        if (m.aggregation === "sum") return 0;
        if (m.aggregation === "avg") return { sum: 0, n: 0 };
        if (m.aggregation === "min") return null;
        if (m.aggregation === "max") return null;
        return 0;
      });

      groups.set(key, { groupVals, state: initState });
    }

    const entry = groups.get(key);

    measures.forEach((m, i) => {
      const agg = m.aggregation;
      const cid = m.column_id;

      if (agg === "count") {
        entry.state[i] += 1;
        return;
      }

      const raw = cid === "*" ? null : getCell(row, cid, colIndex);
      const num = typeof raw === "number" ? raw : NaN;

      if (agg === "sum") {
        if (!Number.isNaN(num)) entry.state[i] += num;
        return;
      }

      if (agg === "avg") {
        if (!Number.isNaN(num)) {
          entry.state[i].sum += num;
          entry.state[i].n += 1;
        }
        return;
      }

      if (agg === "min") {
        if (!Number.isNaN(num)) {
          entry.state[i] = entry.state[i] === null ? num : Math.min(entry.state[i], num);
        }
        return;
      }

      if (agg === "max") {
        if (!Number.isNaN(num)) {
          entry.state[i] = entry.state[i] === null ? num : Math.max(entry.state[i], num);
        }
        return;
      }
    });
  }

  const out = [];
  for (const { groupVals, state } of groups.values()) {
    const finalized = state.map((s, i) => {
      const agg = measures[i].aggregation;
      if (agg === "avg") return s.n ? round2(s.sum / s.n) : null;
      if (agg === "count") return s; // keep integer
      return round2(s);
    });

    out.push([...groupVals, ...finalized]);
  }

  return out;
}

/** -------------------------
 * Middleware (CORS + logging)
 * ------------------------*/
app.use((req, res, next) => {
  res.header("Access-Control-Allow-Origin", "*");
  res.header(
    "Access-Control-Allow-Headers",
    "Origin, X-Requested-With, Content-Type, Accept, X-Secret, X-Host, X-Token, X-Key, ngrok-skip-browser-warning"
  );
  res.header("Access-Control-Allow-Methods", "GET,POST,OPTIONS");
  if (req.method === "OPTIONS") return res.sendStatus(200);
  next();
});

app.use((req, _res, next) => {
  const ts = new Date().toISOString();
  console.log(`[${ts}] ${req.method} ${req.path}`);
  next();
});

/** -------------------------
 * Secret management
 * ------------------------*/
const ENV = process.env.NODE_ENV || "development";
const SECRET =
  process.env.LUZMO_PLUGIN_SECRET ??
  (ENV === "development" ? "dev_secret" : undefined);

if (!SECRET) {
  console.error("❌ LUZMO_PLUGIN_SECRET is not set. Set it and restart.");
  process.exit(1);
}

console.log(`✓ Using secret: ${ENV === "development" ? SECRET : "***"}`);
console.log(`✓ Environment: ${ENV}`);

function checkSecret(req, res) {
  const secret = req.header("X-Secret");
  if (!secret) {
    res.status(401).json({
      type: { code: 401, description: "Unauthorized" },
      message: "Missing X-Secret header",
    });
    return false;
  }
  if (secret !== SECRET) {
    res.status(401).json({
      type: { code: 401, description: "Unauthorized" },
      message: "Invalid X-Secret",
    });
    return false;
  }
  return true;
}

/** -------------------------
 * Data + dataset definition
 * ------------------------*/
function generateData({
  days = 120,
  categories = ["A", "B", "C", "D", "E"],
  seed = 42,
} = {}) {
  let x = seed;
  const rnd = () => (x = (x * 1664525 + 1013904223) % 4294967296) / 4294967296;

  const start = new Date("2025-01-01T00:00:00.000Z");
  const rows = [];

  for (let d = 0; d < days; d++) {
    for (const c of categories) {
      const date = new Date(start);
      date.setUTCDate(start.getUTCDate() + d);

      const n = 1 + Math.floor(rnd() * 5);

      for (let i = 0; i < n; i++) {
        const base = 5 + rnd() * 50;
        const seasonal = 10 * Math.sin((2 * Math.PI * d) / 30);
        const value = Math.max(
          0,
          Math.round((base + seasonal + rnd() * 5) * 100) / 100
        );

        // [category, dateISO, value]
        rows.push([c, date.toISOString(), value]);
      }
    }
  }
  return rows;
}

const data = generateData({
  days: 180,
  categories: ["A", "B", "C", "D", "E", "F"],
  seed: 123,
});

const DATASET_ID = "demo";

// Luzmo /datasets schema (localized name objects)
const dataset = {
  id: DATASET_ID,
  name: { en: "Sample Dataset" },
  description: { en: "Demo dataset with category, date and value metrics" },
  properties: {
    row_limit: 100000,
    supports_pushdown: true,
    supports_sorting: true,
  },
  columns: [
    {
      id: "category",
      name: { en: "Category" },
      type: "hierarchy",
      properties: {
        display_name: { en: "Category" },
        filterable: true,
        groupable: true,
      },
    },
    {
      id: "date",
      name: { en: "Date" },
      type: "datetime",
      subtype: "date",
      properties: {
        display_name: { en: "Date" },
        filterable: true,
        groupable: true,
        format: "YYYY-MM-DD",
      },
    },
    {
      id: "value",
      name: { en: "Value" },
      type: "numeric",
      properties: {
        display_name: { en: "Value" },
        filterable: true,
        aggregable: true,
        format: "0.00",
      },
    },
  ],
};

// column indexes for data rows
const colIndex = { category: 0, date: 1, value: 2 };

/** -------------------------
 * Filters
 * ------------------------*/
function applyOptionalFilters(rows, filters) {
  if (!Array.isArray(filters) || filters.length === 0) return rows;

  let result = rows;

  for (const f of filters) {
    const col = resolveColumnId(f);
    const expr = f?.expression;
    const rawVal = f?.value;

    const idx = colIndex[col];
    if (idx === undefined) continue;

    // IS NULL / IS NOT NULL
    if (expr === "is null") {
      result = result.filter((row) => row[idx] === null || row[idx] === undefined);
      continue;
    }
    if (expr === "is not null") {
      result = result.filter((row) => row[idx] !== null && row[idx] !== undefined);
      continue;
    }

    // IN / NOT IN
    if (expr === "in" && Array.isArray(rawVal)) {
      result = result.filter((row) => rawVal.includes(row[idx]));
      continue;
    }
    if ((expr === "not in" || expr === "nin") && Array.isArray(rawVal)) {
      result = result.filter((row) => !rawVal.includes(row[idx]));
      continue;
    }

    // Date comparisons (for "date" column)
    if (col === "date" && (expr === ">=" || expr === ">" || expr === "<=" || expr === "<")) {
      const cmp = toTime(rawVal);
      if (cmp === null) continue;

      result = result.filter((row) => {
        const cellT = toTime(row[idx]);
        if (cellT === null) return false;
        if (expr === ">=") return cellT >= cmp;
        if (expr === ">") return cellT > cmp;
        if (expr === "<=") return cellT <= cmp;
        if (expr === "<") return cellT < cmp;
        return true;
      });
      continue;
    }

    // Date BETWEEN: value = [fromISO, toISO]
    if (
      col === "date" &&
      (expr === "between" || expr === "in range") &&
      Array.isArray(rawVal) &&
      rawVal.length === 2
    ) {
      const from = toTime(rawVal[0]);
      const to = toTime(rawVal[1]);
      if (from === null || to === null) continue;

      result = result.filter((row) => {
        const cellT = toTime(row[idx]);
        return cellT !== null && cellT >= from && cellT <= to;
      });
      continue;
    }

    // EQUALS (= or ==)
    if ((expr === "=" || expr === "==") && !Array.isArray(rawVal)) {
      result = result.filter((row) => row[idx] === rawVal);
      continue;
    }

    // NOT EQUALS (!= or !==)
    if ((expr === "!=" || expr === "!==") && !Array.isArray(rawVal)) {
      result = result.filter((row) => row[idx] !== rawVal);
      continue;
    }

    // Numeric comparisons (>=, >, <, <=)
    if (expr === ">=" || expr === ">" || expr === "<" || expr === "<=") {
      const v = normalizeValue(rawVal);
      if (typeof v !== "number") continue;

      result = result.filter((row) => {
        const cell = row[idx];
        if (typeof cell !== "number") return false;
        if (expr === ">=") return cell >= v;
        if (expr === ">") return cell > v;
        if (expr === "<=") return cell <= v;
        if (expr === "<") return cell < v;
        return true;
      });
      continue;
    }

    // String contains / like
    if ((expr === "contains" || expr === "like") && typeof rawVal === "string") {
      const needle = rawVal.toLowerCase();
      result = result.filter((row) =>
        String(row[idx] ?? "").toLowerCase().includes(needle)
      );
      continue;
    }

    // Case-insensitive like
    if ((expr === "ilike" || expr === "similar") && typeof rawVal === "string") {
      const needle = rawVal.toLowerCase();
      result = result.filter((row) =>
        String(row[idx] ?? "").toLowerCase().includes(needle)
      );
      continue;
    }

    // Starts with
    if ((expr === "starts_with" || expr === "starts with") && typeof rawVal === "string") {
      const needle = rawVal.toLowerCase();
      result = result.filter((row) =>
        String(row[idx] ?? "").toLowerCase().startsWith(needle)
      );
      continue;
    }

    // Ends with
    if ((expr === "ends_with" || expr === "ends with") && typeof rawVal === "string") {
      const needle = rawVal.toLowerCase();
      result = result.filter((row) =>
        String(row[idx] ?? "").toLowerCase().endsWith(needle)
      );
      continue;
    }

    // Unknown expressions: ignore safely
  }

  return result;
}

/** -------------------------
 * Routes
 * ------------------------*/
// /authorize
app.post("/authorize", (req, res) => {
  if (!checkSecret(req, res)) return;
  return res.status(200).json({ ok: true });
});

// /datasets (GET + POST)
function datasetsHandler(req, res) {
  if (!checkSecret(req, res)) return;
  return res.status(200).json([dataset]);
}
app.get("/datasets", datasetsHandler);
app.post("/datasets", datasetsHandler);

// query
function executeQuery(rawData, columns, filters, limit, options = {}) {
  const pushdown = options.pushdown === true;
  const sort = options.sort;

  // Determine aggregation vs raw mode
  const hasAgg = Array.isArray(columns) && columns.some((c) => !!c?.aggregation);
  const groupByCols = (columns ?? []).filter((c) => !c?.aggregation);
  const measures = (columns ?? [])
    .filter((c) => !!c?.aggregation)
    .map((c) => ({
      column_id: colId(c),
      aggregation: c.aggregation,
    }));

  // Apply filters
  let result = applyOptionalFilters(rawData, filters);

  // Consolidate aggregation logic
  if (pushdown && hasAgg) {
    result = aggregateRows(result, groupByCols, measures, colIndex);
    result = applySorting(result, sort, groupByCols, measures);
  } else {
    // Raw mode: project columns first, then sort
    result = projectRows(result, columns, colIndex);
    if (Array.isArray(sort)) {
      result = applySortingRaw(result, sort);
    }
  }

  // Apply limit
  if (typeof limit === "number" && limit > 0) {
    result = result.slice(0, limit);
  }

  return result;
}

/** Sort aggregated results */
function applySorting(rows, sortSpec, groupByCols, measures) {
  if (!Array.isArray(sortSpec) || sortSpec.length === 0) {
    // Default: auto-detect based on first group-by column
    const firstGb = groupByCols[0];
    const firstGbId = colId(firstGb);

    if (firstGbId === "date" || firstGb?.type === "datetime") {
      // Time-series: ascending
      return rows.sort((a, b) => Date.parse(a[0]) - Date.parse(b[0]));
    } else {
      // Default: descending on first measure
      const measureStart = groupByCols.length;
      return rows.sort((a, b) => (Number(b[measureStart]) || 0) - (Number(a[measureStart]) || 0));
    }
  }

  // Apply specified sorts
  return rows.sort((a, b) => {
    for (const s of sortSpec) {
      const colId_ = resolveColumnId(s);
      const idx = findColumnIndex(colId_, groupByCols, measures);
      if (idx === -1) continue;

      const aVal = a[idx];
      const bVal = b[idx];
      const dir = s?.direction === "asc" || s?.order === "asc" ? 1 : -1;

      if (typeof aVal === "number" && typeof bVal === "number") {
        const cmp = aVal - bVal;
        if (cmp !== 0) return cmp * dir;
      } else if (typeof aVal === "string" && typeof bVal === "string") {
        const cmp = aVal.localeCompare(bVal);
        if (cmp !== 0) return cmp * dir;
      } else if (aVal instanceof Date || bVal instanceof Date) {
        const aMsec = new Date(aVal).getTime();
        const bMsec = new Date(bVal).getTime();
        const cmp = aMsec - bMsec;
        if (cmp !== 0) return cmp * dir;
      }
    }
    return 0;
  });
}

/** Sort raw (projected) results */
function applySortingRaw(rows, sortSpec) {
  if (!Array.isArray(sortSpec) || sortSpec.length === 0) return rows;

  return rows.sort((a, b) => {
    for (const s of sortSpec) {
      const idx = s?.column_index || s?.index || 0;
      if (idx < 0 || idx >= a.length) continue;

      const aVal = a[idx];
      const bVal = b[idx];
      const dir = s?.direction === "asc" || s?.order === "asc" ? 1 : -1;

      if (typeof aVal === "number" && typeof bVal === "number") {
        const cmp = aVal - bVal;
        if (cmp !== 0) return cmp * dir;
      } else if (typeof aVal === "string" && typeof bVal === "string") {
        const cmp = aVal.localeCompare(bVal);
        if (cmp !== 0) return cmp * dir;
      }
    }
    return 0;
  });
}

/** Find column index in aggregated output */
function findColumnIndex(colId_, groupByCols, measures) {
  // Check group-by columns first
  for (let i = 0; i < groupByCols.length; i++) {
    if (colId(groupByCols[i]) === colId_) return i;
  }
  // Then measures
  for (let i = 0; i < measures.length; i++) {
    if (measures[i].column_id === colId_) return groupByCols.length + i;
  }
  return -1;
}

// /query
app.post("/query", (req, res) => {
  if (!checkSecret(req, res)) return;

  const rid = makeReqId();

  if (!req.body || typeof req.body.id !== "string") {
    return res.status(400).json({
      type: { code: 400, description: "Bad Request" },
      message: "Missing or invalid dataset id in request body",
    });
  }

  const { id, filters, limit, columns, options } = req.body;
  const includeMetadata = options?.include_metadata === true;

  console.log(
    `[${rid}] /query id=${id} limit=${limit ?? "-"} cols=${Array.isArray(columns) ? columns.length : 0}`
  );

  if (limit !== undefined && (typeof limit !== "number" || limit < 1)) {
    return res.status(400).json({
      type: { code: 400, description: "Bad Request" },
      message: "Limit must be a positive number",
    });
  }

  if (id !== DATASET_ID) {
    return res.status(404).json({
      type: { code: 404, description: "Not Found" },
      message: `Unknown dataset id: ${id}`,
    });
  }

  try {
    const rows = executeQuery(data, columns, filters, limit, options);
    console.log(`[${rid}] ✓ rows_out=${rows.length}`);

    // Optional: return with metadata envelope
    if (includeMetadata) {
      const colMetadata = (columns ?? []).map((c) => {
        const colDef = dataset.columns.find((dc) => colId(dc) === colId(c));
        return {
          id: colId(c),
          name: colDef?.name,
          type: colDef?.type,
          aggregation: c?.aggregation,
          properties: colDef?.properties,
        };
      });

      return res.status(200).json({
        columns: colMetadata,
        rows: rows,
        rowCount: rows.length,
        properties: dataset.properties,
      });
    }

    return res.status(200).json(rows);
  } catch (error) {
    console.error(`[${rid}] ❌ Query error: ${error?.message || error}`);
    return res.status(500).json({
      type: { code: 500, description: "Internal Server Error" },
      message: "An error occurred while processing the query",
    });
  }
});

// Health
app.get("/", (_req, res) => res.status(200).send("OK"));
app.get("/health", (_req, res) => {
  res.status(200).json({ ok: true, timestamp: new Date().toISOString() });
});

// Global error handler
app.use((err, _req, res, _next) => {
  console.error(`❌ Unhandled error: ${err?.message || err}`);
  res.status(500).json({
    type: { code: 500, description: "Internal Server Error" },
    message: ENV === "development" ? String(err?.message || err) : "An error occurred",
  });
});

// 404
app.use((req, res) => {
  res.status(404).json({
    type: { code: 404, description: "Not Found" },
    message: `Endpoint ${req.method} ${req.path} not found`,
  });
});

const port = process.env.PORT || 3000;
app.listen(port, "0.0.0.0", () => {
  console.log(`✓ Server running on http://localhost:${port}`);
});
