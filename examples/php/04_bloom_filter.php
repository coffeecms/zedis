<?php
require 'vendor/autoload.php';

$client = new Predis\Client([
    'host' => '127.0.0.1',
    'port' => 6379,
]);

echo "--- 04 Bloom Filter ---\n";
$key = "bad_ips";

// BF.ADD
$client->executeRaw(['BF.ADD', $key, '192.168.1.55']);
$client->executeRaw(['BF.ADD', $key, '10.0.0.1']);

// BF.EXISTS
$check = '192.168.1.55';
$exists = $client->executeRaw(['BF.EXISTS', $key, $check]);
echo "Is '$check' bad? " . ($exists ? "Yes" : "No") . "\n";

$safe = '8.8.8.8';
$existsSafe = $client->executeRaw(['BF.EXISTS', $key, $safe]);
echo "Is '$safe' bad? " . ($existsSafe ? "Yes" : "No") . "\n";
?>