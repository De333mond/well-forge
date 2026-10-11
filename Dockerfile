FROM node:24-bookworm-slim

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

EXPOSE 1420

CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]
