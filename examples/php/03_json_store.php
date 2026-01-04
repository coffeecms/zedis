<?php
require 'vendor/autoload.php';

$client = new Predis\Client([
    'host' => '127.0.0.1',
    'port' => 6379,
]);

echo "--- 03 JSON Document Store ---\n";

$profile = [
    "username" => "dev_jane",
    "active" => true,
    "roles" => ["admin", "editor"]
];

// JSON.SET
echo "Storing profile...\n";
$client->executeRaw(['JSON.SET', 'user:500', json_encode($profile)]);

// JSON.GET
echo "Retrieving profile...\n";
$json = $client->executeRaw(['JSON.GET', 'user:500', '.']);
$data = json_decode($json, true);

echo "Username: " . $data['username'] . "\n";
?>