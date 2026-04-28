<?php
/**
 * Plugin Name: FeelPay Gateway
 * Description: WooCommerce payment gateway for FeelPay.
 * Version: 0.1.0
 */

if (!defined('ABSPATH')) {
    exit;
}

add_filter('woocommerce_payment_gateways', function ($gateways) {
    $gateways[] = 'WC_Gateway_FeelPay';
    return $gateways;
});

add_action('plugins_loaded', function () {
    if (!class_exists('WC_Payment_Gateway')) {
        return;
    }

    class WC_Gateway_FeelPay extends WC_Payment_Gateway
    {
        public function __construct()
        {
            $this->id = 'feelpay';
            $this->method_title = 'FeelPay';
            $this->method_description = 'Accept UPI, cards, net banking, and wallets via FeelPay.';
            $this->has_fields = false;
            $this->init_form_fields();
            $this->init_settings();
            $this->title = $this->get_option('title', 'FeelPay');
            add_action('woocommerce_update_options_payment_gateways_' . $this->id, [$this, 'process_admin_options']);
        }

        public function init_form_fields()
        {
            $this->form_fields = [
                'enabled' => ['title' => 'Enable', 'type' => 'checkbox', 'label' => 'Enable FeelPay', 'default' => 'no'],
                'title' => ['title' => 'Title', 'type' => 'text', 'default' => 'FeelPay'],
                'api_key' => ['title' => 'API Key', 'type' => 'password'],
            ];
        }
    }
});
