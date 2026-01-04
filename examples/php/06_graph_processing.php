<?php
require 'vendor/autoload.php';

$client = new Predis\Client([
    'host' => '127.0.0.1',
    'port' => 6379,
]);

echo "--- 06 Graph Processing ---\n";
$key = "org_chart";

// GRAPH.ADD Manager -> Employee
$client->executeRaw(['GRAPH.ADD', $key, 'CEO', 'CTO']);
$client->executeRaw(['GRAPH.ADD', $key, 'CTO', 'EngManager']);
$client->executeRaw(['GRAPH.ADD', $key, 'EngManager', 'Developer']);

// GRAPH.BFS
echo "Org Chart below CEO (Depth 3):\n";
$results = $client->executeRaw(['GRAPH.BFS', $key, 'CEO', '3']);
print_r($results);
?>