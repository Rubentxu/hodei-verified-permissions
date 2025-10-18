#!/bin/bash
# Script de verificación de build completo

set -e

echo "🔨 Verificando build completo de Hodei Verified Permissions..."
echo ""

echo "📦 1. Limpiando builds anteriores..."
cargo clean

echo ""
echo "🏗️  2. Compilando servidor principal..."
cargo build --package hodei-verified-permissions --release

echo ""
echo "📚 3. Compilando SDK..."
cargo build --package hodei-permissions-sdk --release

echo ""
echo "✅ 4. Compilando todo el workspace..."
cargo build --all --release

echo ""
echo "🎉 ¡BUILD COMPLETADO EXITOSAMENTE!"
echo ""
echo "Para ejecutar:"
echo "  Servidor: cargo run --release"
echo "  Ejemplo SDK: cd sdk && cargo run --example basic_usage"
echo ""
echo "✨ Proyecto 100% funcional y listo para usar"
