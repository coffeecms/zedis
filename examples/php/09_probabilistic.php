<?php
require 'vendor/autoload.php';

$client = new Predis\Client([
    'scheme' => 'tcp',
    'host'   => '127.0.0.1',
    'port'   => 6379,
]);

echo "--- 09 God Tier Probabilistic Data Structures ---\n";

// 1. HyperLogLog
$client->executeRaw(['PFADD', 'hll_php', 'p1', 'p2']);
$count = $client->executeRaw(['PFCOUNT', 'hll_php']);
echo "[HLL] Count: $count\n";

// 2. Cuckoo Filter
$client->executeRaw(['CF.ADD', 'cf_php', 'test']);
$exists = $client->executeRaw(['CF.EXISTS', 'cf_php', 'test']);
echo "[Cuckoo] Exists: $exists\n";

// 3. Count-Min Sketch
$client->executeRaw(['CMS.INCRBY', 'cms_php', 'page_view', 5]);
$val = $client->executeRaw(['CMS.QUERY', 'cms_php', 'page_view']);
echo "[CMS] Count: $val\n";

// 4. Top-K
$client->executeRaw(['TOPK.ADD', 'topk_php', 'dog', 'cat', 'dog']);
$list = $client->executeRaw(['TOPK.LIST', 'topk_php']);
echo "[TopK] List: " . json_encode($list) . "\n";

// 5. t-digest
$client->executeRaw(['TDIGEST.ADD', 'td_php', 12.5]);
$res = $client->executeRaw(['TDIGEST.QUANTILE', 'td_php', 0.5]);
echo "[t-digest] Median: $res\n";
?>
