<?php
require 'vendor/autoload.php';

$client = new Predis\Client([
    'host' => '127.0.0.1',
    'port' => 6379,
]);

echo "--- 05 Time Series ---\n";
$key = "cpu_usage";

$now = time() * 1000; // Milliseconds

// TS.ADD
$client->executeRaw(['TS.ADD', $key, $now, '15']);
$client->executeRaw(['TS.ADD', $key, $now + 1000, '20']);
$client->executeRaw(['TS.ADD', $key, $now + 2000, '18']);

// TS.RANGE
echo "Fetching metrics...\n";
$data = $client->executeRaw(['TS.RANGE', $key, $now, $now + 5000]);
print_r($data);
?>