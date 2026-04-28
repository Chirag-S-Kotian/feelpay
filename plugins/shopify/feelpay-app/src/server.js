import express from 'express';

const app = express();
app.use(express.json());

app.get('/health', (_req, res) => res.json({ ok: true }));
app.post('/webhooks/orders-paid', (req, res) => {
  console.log('Shopify order paid', req.body?.id);
  res.sendStatus(204);
});

app.listen(process.env.PORT || 3000);
