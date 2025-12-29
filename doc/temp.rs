use anyhow::Result;
use polars::prelude::*;

// Структура входных данных
#[derive(Debug, Clone)]
struct RawPeak {
    mass_to_charge: f32,
    signal: u16,
}

// Структура для хранения найденных характеристик
#[derive(Debug)]
struct SpectrumFeatures {
    base_peak_mz: f32,
    molecular_ion_candidate: f32,
    i_74: f32,  // McLafferty (Saturated)
    i_87: f32,  // Saturated series
    i_55: f32,  // Monoene base
    i_67: f32,  // Diene base
    i_79: f32,  // Polyene base
    i_91: f32,  // Tropylium (Polyene)
    i_108: f32, // Omega-3 marker
    i_150: f32, // Omega-6 marker
}

fn main() -> Result<()> {
    // 1. Пример данных (имитация спектра, например, EPA - 20:5 n-3)
    // В реальности вы загрузите это из вашего источника
    let raw_data = vec![
        RawPeak {
            mass_to_charge: 55.0,
            signal: 600,
        },
        RawPeak {
            mass_to_charge: 67.0,
            signal: 400,
        },
        RawPeak {
            mass_to_charge: 74.0,
            signal: 50,
        }, // Маленький для полиенов
        RawPeak {
            mass_to_charge: 79.0,
            signal: 1000,
        }, // Base Peak для полиенов
        RawPeak {
            mass_to_charge: 91.0,
            signal: 800,
        }, // Тропилий
        RawPeak {
            mass_to_charge: 108.0,
            signal: 300,
        }, // Omega-3 маркер
        RawPeak {
            mass_to_charge: 150.0,
            signal: 10,
        }, // Omega-6 (нет)
        RawPeak {
            mass_to_charge: 180.0,
            signal: 150,
        }, // Alpha маркер (Delta 5)
        RawPeak {
            mass_to_charge: 316.0,
            signal: 5,
        }, // M+ (очень маленький)
    ];

    // 2. Создание DataFrame
    let df = create_dataframe(&raw_data)?;

    // 3. Нормализация и анализ
    let features = analyze_spectrum(&df)?;

    println!("--- Spectrum Features ---");
    println!("{:#?}", features);

    // 4. Классификация
    let classification = classify(&features);
    println!("\n--- Classification Result ---");
    println!("{}", classification);

    Ok(())
}

fn create_dataframe(data: &[RawPeak]) -> Result<DataFrame> {
    let mz: Vec<f32> = data.iter().map(|p| p.mass_to_charge).collect();
    let signal: Vec<u16> = data.iter().map(|p| p.signal).collect();

    let df = DataFrame::new(vec![Series::new("mz", mz), Series::new("signal", signal)])?;

    Ok(df)
}

fn analyze_spectrum(df: &DataFrame) -> Result<SpectrumFeatures> {
    // Нормализация: Находим макс сигнал и приводим к 1000
    let max_signal = df.column("signal")?.max::<u16>().unwrap_or(1) as f32;

    let df_norm = df
        .clone()
        .lazy()
        .with_column(
            (col("signal").cast(DataType::Float32) / lit(max_signal) * lit(1000.0))
                .alias("norm_intensity"),
        )
        .collect()?;

    // Поиск Base Peak m/z
    let base_peak_mask = df_norm.column("norm_intensity")?.equal(1000.0)?;
    let base_peak_mz = df_norm
        .filter(&base_peak_mask)?
        .column("mz")?
        .f32()?
        .get(0)
        .unwrap_or(0.0);

    // Поиск кандидата на Молекулярный ион (самая большая масса с сигналом > 1% от базы)
    // В статьях сказано, что у полиенов M+ очень мал, поэтому порог низкий (10 из 1000)
    let m_plus_mask = df_norm.column("norm_intensity")?.gt(10.0)?;
    let m_plus_candidate = df_norm
        .filter(&m_plus_mask)?
        .column("mz")?
        .max::<f32>()
        .unwrap_or(0.0);

    // Хелпер для извлечения интенсивности конкретного пика с допуском +/- 0.5
    let get_intensity = |target_mz: f32| -> f32 {
        let mask = df_norm
            .column("mz")
            .unwrap()
            .f32()
            .unwrap()
            .into_iter()
            .map(|opt_v| opt_v.map(|v| (v - target_mz).abs() < 0.5).unwrap_or(false))
            .collect::<BooleanChunked>();

        if let Ok(filtered) = df_norm.filter(&mask) {
            if filtered.height() > 0 {
                // Если попало несколько точек, берем максимум
                return filtered
                    .column("norm_intensity")
                    .unwrap()
                    .max::<f32>()
                    .unwrap_or(0.0);
            }
        }
        0.0
    };

    Ok(SpectrumFeatures {
        base_peak_mz,
        molecular_ion_candidate: m_plus_candidate,
        i_74: get_intensity(74.0),
        i_87: get_intensity(87.0),
        i_55: get_intensity(55.0),
        i_67: get_intensity(67.0),
        i_79: get_intensity(79.0),
        i_91: get_intensity(91.0),
        i_108: get_intensity(108.0),
        i_150: get_intensity(150.0),
    })
}

fn classify(f: &SpectrumFeatures) -> String {
    let mut report = String::new();

    // --- Логика из статей ---

    // 1. Проверка на Насыщенные (Saturated)
    // Критерий: Базовый пик 74, есть 87
    if (f.base_peak_mz - 74.0).abs() < 0.5 || (f.i_74 > 800.0) {
        if f.i_87 > 100.0 {
            return "SATURATED Fatty Acid Methyl Ester (FAME). \nCriteria: Base peak at m/z 74 (McLafferty) and significant m/z 87.".to_string();
        }
    }

    // 2. Проверка на Моноеновые (Monoenoic)
    // Критерий: Базовый пик 55, пик 74 есть, но не доминирует.
    if (f.base_peak_mz - 55.0).abs() < 0.5 {
        // Дополнительная проверка: потеря метанола [M-32] (здесь упрощенно без расчета M)
        return "MONOENOIC FAME (Likely). \nCriteria: Base peak at m/z 55. Hydrocarbon ions dominate.".to_string();
    }

    // 3. Проверка на Диеновые (Dienoic)
    // Критерий: Базовый пик 67 или 81
    if (f.base_peak_mz - 67.0).abs() < 0.5 || (f.base_peak_mz - 81.0).abs() < 0.5 {
        return "DIENOIC FAME (Likely). \nCriteria: Base peak at m/z 67/81.".to_string();
    }

    // 4. Проверка на Полиеновые (Polyenoic)
    // Критерий: Базовый пик 79, наличие 91 (тропилий), маленький 74
    if (f.base_peak_mz - 79.0).abs() < 0.5 {
        report.push_str("POLYENOIC FAME detected.\n");
        report.push_str("Criteria: Base peak at m/z 79. Low m/z 74 intensity.\n");

        if f.i_91 > 200.0 {
            report.push_str("-> Tropylium ion (m/z 91) present: confirms high unsaturation.\n");
        }

        // Проверка Omega-маркеров
        let mut omega_found = false;
        if f.i_108 > 100.0 {
            // Порог чувствительности
            report.push_str("-> Diagnostic Ion m/z 108 found: Suggests Omega-3 (n-3) family.\n");
            omega_found = true;
        }
        if f.i_150 > 100.0 {
            report.push_str("-> Diagnostic Ion m/z 150 found: Suggests Omega-6 (n-6) family.\n");
            omega_found = true;
        }

        if !omega_found {
            report.push_str(
                "-> No specific Omega-3/6 markers (108/150) found with high intensity.\n",
            );
        }

        return report;
    }

    "Unknown / Unclassified based on current criteria".to_string()
}
