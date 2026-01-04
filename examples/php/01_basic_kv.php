<?php
require 'vendor/autoload.php';

$client = new Predis\Client([
    'scheme' => 'tcp',
    'host'   => '127.0.0.1',
    'port'   => 6379,
]);

echo "--- 01 Basic Key-Value Operations ---\n";

// SET
echo "Setting key 'framework' to 'Laravel'...\n";
$client->set('framework', 'Laravel');

// GET
$value = $client->get('framework');
echo "Got value: $value\n";

// EXPIRY
echo "Setting key 'cache:home' with 60s expiry...\n";
$client->setex('cache:home', 60, '<html>...</html>');

// PING
$pong = $client->ping();
echo "PING: $pong\n";
?>
