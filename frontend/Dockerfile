FROM --platform=linux/arm64 node:20-alpine AS builder

WORKDIR /app

COPY package*.json ./

RUN npm ci --no-fund --no-audit

COPY . .

RUN npm run build

FROM --platform=linux/arm64 node:20-alpine AS runner

WORKDIR /app

ENV NODE_ENV=production

RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nuxt

COPY --from=builder /app/package*.json ./
COPY --from=builder /app/.output ./.output

RUN chown -R nuxt:nodejs .output

USER nuxt

EXPOSE 3000

ENV NUXT_HOST=0.0.0.0
ENV NUXT_PORT=3000

CMD ["node", ".output/server/index.mjs"]
