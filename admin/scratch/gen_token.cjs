const jwt = require('jsonwebtoken');
const secret = 'feelpay_jwt_secret';
const merchant_id = 'cd317088-a9ef-495c-ab3e-2a88603c54ac';
const payload = {
  sub: merchant_id,
  merchant_id: merchant_id,
  role: 'Super Admin',
  permissions: ['*'],
  iat: Math.floor(Date.now() / 1000),
  exp: Math.floor(Date.now() / 1000) + (60 * 60)
};
const token = jwt.sign(payload, secret);
console.log(token);
